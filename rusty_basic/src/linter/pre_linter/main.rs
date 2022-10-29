use crate::linter::pre_linter::const_rules::global_const;
use crate::linter::pre_linter::sub_program_context::{FunctionContext, SubContext, ToSignature};
use crate::linter::pre_linter::{ConstantMap, PreLinterResult, ResolvedParamTypes};
use crate::linter::type_resolver::IntoTypeQualifier;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::ResolvedParamType;
use crate::parser::*;
use rusty_common::*;

pub struct MainContext {
    resolver: TypeResolverImpl,
    user_defined_types: UserDefinedTypes,
    functions: FunctionContext,
    subs: SubContext,
    global_constants: ConstantMap,
    declaration_pos: Location,
}

pub fn pre_lint_program(program: &ProgramNode) -> Result<PreLinterResult, QErrorNode> {
    let mut ctx = MainContext {
        resolver: TypeResolverImpl::new(),
        user_defined_types: Default::default(),
        functions: FunctionContext::new(),
        subs: SubContext::new(),
        global_constants: Default::default(),
        declaration_pos: Location::start(),
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
    fn on_program(&mut self, program: &ProgramNode) -> Result<(), QErrorNode> {
        for Locatable { element, pos } in program {
            self.declaration_pos = *pos;
            match element {
                TopLevelToken::DefType(def_type) => {
                    self.on_def_type(def_type);
                }
                TopLevelToken::FunctionDeclaration(name, params) => {
                    self.on_function_declaration(name, params)?;
                }
                TopLevelToken::FunctionImplementation(f) => {
                    self.on_function_implementation(f)?;
                }
                TopLevelToken::Statement(s) => {
                    self.on_statement(s)?;
                }
                TopLevelToken::SubDeclaration(name, params) => {
                    self.on_sub_declaration(name, params)?;
                }
                TopLevelToken::SubImplementation(s) => {
                    self.on_sub_implementation(s)?;
                }
                TopLevelToken::UserDefinedType(user_defined_type) => {
                    self.on_user_defined_type(user_defined_type)
                        .patch_err_pos(pos)?;
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
        name: &NameNode,
        params: &ParamNameNodes,
    ) -> Result<(), QErrorNode> {
        let param_types: ResolvedParamTypes = self.on_param_name_nodes(params)?;
        let bare_name = name.as_ref().bare_name();
        let signature = name.as_ref().to_signature(&self.resolver, param_types);
        self.functions
            .add_declaration(bare_name, signature, self.declaration_pos)
            .with_err_at(name.pos())
    }

    fn on_function_implementation(&mut self, f: &FunctionImplementation) -> Result<(), QErrorNode> {
        let FunctionImplementation { name, params, .. } = f;
        let param_types: ResolvedParamTypes = self.on_param_name_nodes(params)?;
        let bare_name = name.as_ref().bare_name();
        let signature = name.as_ref().to_signature(&self.resolver, param_types);
        self.functions
            .add_implementation(bare_name, signature, self.declaration_pos)
            .with_err_at(name.pos())
    }

    fn on_statement(&mut self, s: &Statement) -> Result<(), QErrorNode> {
        match s {
            Statement::Const(name, expr) => self.on_const(name, expr),
            _ => Ok(()),
        }
    }

    fn on_sub_declaration(
        &mut self,
        name: &BareNameNode,
        params: &ParamNameNodes,
    ) -> Result<(), QErrorNode> {
        let param_types: ResolvedParamTypes = self.on_param_name_nodes(params)?;
        let bare_name = name.as_ref();
        let signature = name.as_ref().to_signature(&self.resolver, param_types);
        self.subs
            .add_declaration(bare_name, signature, self.declaration_pos)
            .with_err_at(name.pos())
    }

    fn on_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        let SubImplementation { name, params, .. } = s;
        let param_types: ResolvedParamTypes = self.on_param_name_nodes(params)?;
        let bare_name = name.as_ref();
        let signature = name.as_ref().to_signature(&self.resolver, param_types);
        self.subs
            .add_implementation(bare_name, signature, self.declaration_pos)
            .with_err_at(name.pos())
    }

    fn on_user_defined_type(
        &mut self,
        user_defined_type: &UserDefinedType,
    ) -> Result<(), QErrorNode> {
        super::user_defined_type_rules::user_defined_type(
            &mut self.user_defined_types,
            &self.global_constants,
            user_defined_type,
        )
    }

    fn on_const(&mut self, name: &NameNode, expr: &ExpressionNode) -> Result<(), QErrorNode> {
        global_const(&mut self.global_constants, name, expr)
    }

    fn on_param_name_nodes(
        &self,
        param_name_nodes: &ParamNameNodes,
    ) -> Result<ResolvedParamTypes, QErrorNode> {
        param_name_nodes
            .iter()
            .map(|p| self.on_param_name_node(p))
            .collect()
    }

    fn on_param_name_node(
        &self,
        param_name_node: &ParamNameNode,
    ) -> Result<ResolvedParamType, QErrorNode> {
        self.on_param_name(param_name_node.as_ref())
            .with_err_at(param_name_node)
    }

    fn on_param_name(&self, param_name: &ParamName) -> Result<ResolvedParamType, QError> {
        let bare_name = param_name.bare_name();
        self.resolve_param_type(bare_name, param_name.var_type())
    }

    fn resolve_param_type(
        &self,
        bare_name: &BareName,
        param_type: &ParamType,
    ) -> Result<ResolvedParamType, QError> {
        match param_type {
            ParamType::Bare => {
                let q = bare_name.qualify(&self.resolver);
                Ok(ResolvedParamType::BuiltIn(q, BuiltInStyle::Compact))
            }
            ParamType::BuiltIn(q, built_in_style) => {
                Ok(ResolvedParamType::BuiltIn(*q, *built_in_style))
            }
            ParamType::UserDefined(u) => {
                let type_name: &BareName = u.as_ref();
                if self.user_defined_types.contains_key(type_name) {
                    Ok(ResolvedParamType::UserDefined(type_name.clone()))
                } else {
                    Err(QError::TypeNotDefined)
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
