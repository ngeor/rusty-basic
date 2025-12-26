use crate::core::*;
use crate::pre_linter::sub_program_context::SubprogramContext;
use crate::pre_linter::{ConstantMap, PreLinterResult};
use rusty_common::*;
use rusty_parser::*;

// CONST -> stored in global_constants
// DEFINT -> stored in resolver
// TYPE ... END TYPE -> stored in user_defined_types depends on CONST for resolving string length (AS STRING * some_const)
// FUNCTION/SUB -> depends on resolver for resolving bare names and on user_defined_types to ensure types exist

struct MainContext {
    resolver: TypeResolverImpl,
    user_defined_types: UserDefinedTypes,
    functions: SubprogramContext,
    subs: SubprogramContext,
    global_constants: ConstantMap,
    declaration_pos: Position,
}

pub fn pre_lint_program(program: &Program) -> Result<PreLinterResult, LintErrorPos> {
    let mut ctx = MainContext {
        resolver: TypeResolverImpl::new(),
        user_defined_types: Default::default(),
        functions: SubprogramContext::new(),
        subs: SubprogramContext::new(),
        global_constants: Default::default(),
        declaration_pos: Position::start(),
    };
    <MainContext as Visitor<Program>>::visit(&mut ctx, program)?;
    ctx.post_visit_functions()?;
    ctx.post_visit_subs()?;
    Ok(PreLinterResult::new(
        ctx.functions.implementations(),
        ctx.subs.implementations(),
        ctx.user_defined_types,
    ))
}

impl SetPosition for MainContext {
    fn set_position(&mut self, pos: Position) {
        self.declaration_pos = pos;
    }
}

impl Visitor<DefType> for MainContext {
    fn visit(&mut self, def_type: &DefType) -> VisitResult {
        self.resolver.set(def_type);
        Ok(())
    }
}

impl Visitor<FunctionDeclaration> for MainContext {
    fn visit(&mut self, f: &FunctionDeclaration) -> VisitResult {
        let FunctionDeclaration {
            name: Positioned { element: name, .. },
            parameters: params,
        } = f;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = name.bare_name();
        let q = name.qualify(&self.resolver);
        let signature = Signature::new_function(q, param_types);
        self.functions
            .add_declaration(bare_name.clone(), signature.at_pos(self.declaration_pos))
    }
}

impl Visitor<FunctionImplementation> for MainContext {
    fn visit(&mut self, f: &FunctionImplementation) -> VisitResult {
        let FunctionImplementation {
            name: Positioned { element: name, .. },
            params,
            ..
        } = f;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let bare_name = name.bare_name();
        let q = name.qualify(&self.resolver);
        let signature = Signature::new_function(q, param_types);
        self.functions
            .add_implementation(bare_name.clone(), signature.at_pos(self.declaration_pos))
    }
}

impl Visitor<SubDeclaration> for MainContext {
    fn visit(&mut self, s: &SubDeclaration) -> VisitResult {
        let SubDeclaration {
            name: Positioned {
                element: bare_name, ..
            },
            parameters: params,
        } = s;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let signature = Signature::new_sub(param_types);
        self.subs
            .add_declaration(bare_name.clone(), signature.at_pos(self.declaration_pos))
    }
}

impl Visitor<SubImplementation> for MainContext {
    fn visit(&mut self, s: &SubImplementation) -> VisitResult {
        let SubImplementation {
            name: Positioned {
                element: bare_name, ..
            },
            params,
            ..
        } = s;
        let param_types: ResolvedParamTypes = self.on_parameters(params)?;
        let signature = Signature::new_sub(param_types);
        self.subs
            .add_implementation(bare_name.clone(), signature.at_pos(self.declaration_pos))
    }
}

impl Visitor<Statement> for MainContext {
    fn visit(&mut self, s: &Statement) -> VisitResult {
        match s {
            Statement::Const(c) => self.global_constants.visit(c),
            _ => Ok(()),
        }
    }
}

impl Visitor<UserDefinedType> for MainContext {
    fn visit(&mut self, user_defined_type: &UserDefinedType) -> VisitResult {
        self.delegate().visit(user_defined_type)
    }
}

impl DelegateVisitor<UserDefinedType> for MainContext {
    fn delegate(&mut self) -> impl Visitor<UserDefinedType> {
        super::user_defined_type_visitor::UserDefinedTypeVisitor::new(
            &mut self.user_defined_types,
            self.declaration_pos,
            &self.global_constants,
        )
    }
}

impl ShallowGlobalStatementVisitor for MainContext {}

impl MainContext {
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

    fn post_visit_functions(&self) -> Result<(), LintErrorPos> {
        self.functions.ensure_declarations_are_implemented()?;
        self.functions
            .ensure_does_not_clash_with_built_in(|name| BuiltInFunction::try_parse(name).is_some())
    }

    fn post_visit_subs(&self) -> Result<(), LintErrorPos> {
        // not checking if declarations are present, because in MONEY.BAS there
        // are two SUBs declared but not implemented (and not called either)
        self.subs.ensure_does_not_clash_with_built_in(|name| {
            BuiltInSub::parse_non_keyword_sub(name.as_ref()).is_some()
        })
    }
}
