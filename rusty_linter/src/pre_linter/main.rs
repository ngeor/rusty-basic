use crate::core::IntoTypeQualifier;
use crate::core::TypeResolverImpl;
use crate::core::{LintError, LintErrorPos};
use crate::core::{LintResult, ResolvedParamType};
use crate::pre_linter::const_rules::global_const;
use crate::pre_linter::sub_program_context::{FunctionContext, SubContext, ToSignature};
use crate::pre_linter::{ConstantMap, PreLinterResult, ResolvedParamTypes};
use rusty_common::*;
use rusty_parser::*;

struct MainContext {
    resolver: TypeResolverImpl,
    user_defined_types: UserDefinedTypes,
    functions: FunctionContext,
    subs: SubContext,
    global_constants: ConstantMap,
    declaration_pos: Position,
}

pub fn pre_lint_program(program: &Program) -> Result<PreLinterResult, LintErrorPos> {
    let mut ctx = MainContext {
        resolver: TypeResolverImpl::new(),
        user_defined_types: Default::default(),
        functions: FunctionContext::new(),
        subs: SubContext::new(),
        global_constants: Default::default(),
        declaration_pos: Position::start(),
    };
    ctx.on_program(program)?;
    ctx.functions.post_visit()?;
    ctx.subs.post_visit()?;
    Ok(PreLinterResult::new(
        ctx.functions.implementations(),
        ctx.subs.implementations(),
        ctx.user_defined_types,
    ))
}

// CONST -> stored in global_constants
// DEFINT -> stored in resolver
// TYPE ... END TYPE -> stored in user_defined_types depends on CONST for resolving string length (AS STRING * some_const)
// FUNCTION/SUB -> depends on resolver for resolving bare names and on user_defined_types to ensure types exist

impl MainContext {
    fn on_program(&mut self, program: &Program) -> Result<(), LintErrorPos> {
        for Positioned { element, pos } in program {
            self.declaration_pos = *pos;
            match element {
                GlobalStatement::DefType(def_type) => {
                    self.on_def_type(def_type);
                }
                GlobalStatement::FunctionDeclaration(name, params) => {
                    self.on_function_declaration(name, params)?;
                }
                GlobalStatement::FunctionImplementation(f) => {
                    self.on_function_implementation(f)?;
                }
                GlobalStatement::Statement(s) => {
                    self.on_statement(s)?;
                }
                GlobalStatement::SubDeclaration(name, params) => {
                    self.on_sub_declaration(name, params)?;
                }
                GlobalStatement::SubImplementation(s) => {
                    self.on_sub_implementation(s)?;
                }
                GlobalStatement::UserDefinedType(user_defined_type) => {
                    self.on_user_defined_type(user_defined_type, *pos)?;
                }
            }
        }
        Ok(())
    }

    fn on_def_type(&mut self, def_type: &DefType) {
        self.resolver.set(def_type);
    }

    fn on_function_declaration(
        &mut self,
        name: &NamePos,
        params: &Parameters,
    ) -> Result<(), LintErrorPos> {
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = name.element.bare_name();
        let signature = name.element.to_signature(&self.resolver, param_types);
        self.functions
            .add_declaration(bare_name, signature, self.declaration_pos)
            .with_err_at(name)
    }

    fn on_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), LintErrorPos> {
        let FunctionImplementation { name, params, .. } = f;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = name.element.bare_name();
        let signature = name.element.to_signature(&self.resolver, param_types);
        self.functions
            .add_implementation(bare_name, signature, self.declaration_pos)
            .with_err_at(name)
    }

    fn on_statement(&mut self, s: &Statement) -> Result<(), LintErrorPos> {
        match s {
            Statement::Const(name, expr) => self.on_const(name, expr),
            _ => Ok(()),
        }
    }

    fn on_sub_declaration(
        &mut self,
        name: &BareNamePos,
        params: &Parameters,
    ) -> Result<(), LintErrorPos> {
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = &name.element;
        let signature = bare_name.to_signature(&self.resolver, param_types);
        self.subs
            .add_declaration(bare_name, signature, self.declaration_pos)
            .with_err_at(name)
    }

    fn on_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        let SubImplementation { name, params, .. } = s;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = &name.element;
        let signature = bare_name.to_signature(&self.resolver, param_types);
        self.subs
            .add_implementation(bare_name, signature, self.declaration_pos)
            .with_err_at(name)
    }

    fn on_user_defined_type(
        &mut self,
        user_defined_type: &UserDefinedType,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        super::user_defined_type_rules::user_defined_type(
            &mut self.user_defined_types,
            &self.global_constants,
            user_defined_type,
            pos,
        )
    }

    fn on_const(&mut self, name: &NamePos, expr: &ExpressionPos) -> Result<(), LintErrorPos> {
        global_const(&mut self.global_constants, name, expr)
    }

    fn on_parameters(&self, parameters: &Parameters) -> Result<ResolvedParamTypes, LintErrorPos> {
        parameters
            .iter()
            .map(|p| self.on_parameter_pos(p))
            .collect()
    }

    fn on_parameter_pos(
        &self,
        parameter_pos: &ParameterPos,
    ) -> Result<ResolvedParamType, LintErrorPos> {
        self.on_parameter(&parameter_pos.element)
            .with_err_at(parameter_pos)
    }

    fn on_parameter(&self, parameter: &Parameter) -> Result<ResolvedParamType, LintError> {
        self.resolve_param_type(&parameter.bare_name, &parameter.var_type)
    }

    fn resolve_param_type(
        &self,
        bare_name: &BareName,
        param_type: &ParamType,
    ) -> Result<ResolvedParamType, LintError> {
        match param_type {
            ParamType::Bare => {
                let q = bare_name.qualify(&self.resolver);
                Ok(ResolvedParamType::BuiltIn(q, BuiltInStyle::Compact))
            }
            ParamType::BuiltIn(q, built_in_style) => {
                Ok(ResolvedParamType::BuiltIn(*q, *built_in_style))
            }
            ParamType::UserDefined(u) => {
                let type_name: &BareName = &u.element;
                if self.user_defined_types.contains_key(type_name) {
                    Ok(ResolvedParamType::UserDefined(type_name.clone()))
                } else {
                    Err(LintError::TypeNotDefined)
                }
            }
            ParamType::Array(element_type) => {
                let element_param_type =
                    self.resolve_param_type(bare_name, element_type.as_ref())?;
                Ok(ResolvedParamType::Array(Box::new(element_param_type)))
            }
        }
    }
}
