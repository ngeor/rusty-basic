use rusty_common::*;
use rusty_parser::*;

use crate::core::*;
use crate::pre_linter::ConstantMap;
use crate::pre_linter::sub_program_context::SubprogramContext;

// CONST -> stored in global_constants
// DEFINT -> stored in resolver
// TYPE ... END TYPE -> stored in user_defined_types depends on CONST for resolving string length (AS STRING * some_const)
// FUNCTION/SUB -> depends on resolver for resolving bare names and on user_defined_types to ensure types exist

#[derive(Default)]
struct MainContext {
    resolver: TypeResolverImpl,
    user_defined_types: UserDefinedTypes,
    functions: SubprogramContext,
    subs: SubprogramContext,
    global_constants: ConstantMap,
    declaration_pos: Position,
}

pub fn pre_lint_program(program: &Program) -> Result<LinterContext, LintErrorPos> {
    let mut visitor = GlobalVisitor::new(MainContext::default());
    visitor.visit(program)?;
    let ctx = visitor.delegate();
    ctx.post_visit_functions()?;
    ctx.post_visit_subs()?;
    Ok(LinterContext::new(
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
        let param_types: ResolvedParamTypes = self.resolve_parameters(params)?;
        let bare_name = name.as_bare_name();
        let q = name.qualify(&self.resolver);
        let signature = Signature::Function(param_types, q);
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
        let param_types: ResolvedParamTypes = self.resolve_parameters(params)?;
        let bare_name = name.as_bare_name();
        let q = name.qualify(&self.resolver);
        let signature = Signature::Function(param_types, q);
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
        let param_types: ResolvedParamTypes = self.resolve_parameters(params)?;
        let signature = Signature::Sub(param_types);
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
        let param_types: ResolvedParamTypes = self.resolve_parameters(params)?;
        let signature = Signature::Sub(param_types);
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
        self.user_defined_types_visitor().visit(user_defined_type)
    }
}

impl MainContext {
    fn user_defined_types_visitor(&mut self) -> impl Visitor<UserDefinedType> + use<'_> {
        super::user_defined_type_visitor::UserDefinedTypeVisitor::new(
            &mut self.user_defined_types,
            self.declaration_pos,
            &self.global_constants,
        )
    }

    fn resolve_parameters(
        &mut self,
        parameters: &Parameters,
    ) -> Result<ResolvedParamTypes, LintErrorPos> {
        self.ref_to_value_visit(parameters).map(|v| v.no_pos())
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

impl<T> RefToValueVisitor<T, ResolvedParamType, LintError> for MainContext
where
    T: AsBareName + AsRef<ParamType>,
{
    fn ref_to_value_visit(&mut self, element: &T) -> Result<ResolvedParamType, LintError> {
        let bare_name: &BareName = element.as_bare_name();
        let param_type: &ParamType = element.as_ref();

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
                let temp = RefParamName(bare_name, element_type);
                let element_param_type = self.ref_to_value_visit(&temp)?;
                Ok(ResolvedParamType::Array(Box::new(element_param_type)))
            }
        }
    }
}

/// This is the same as [Parameter],
/// but the members are references.
/// It's needed due to the recursive implementation of `RefToValueVisitor`
/// and the recursive (`Box`) implementation of `ParamType::Array`.
struct RefParamName<'a>(&'a BareName, &'a ParamType);

impl<'a> AsBareName for RefParamName<'a> {
    fn as_bare_name(&self) -> &BareName {
        self.0
    }
}

impl<'a> AsRef<ParamType> for RefParamName<'a> {
    fn as_ref(&self) -> &ParamType {
        self.1
    }
}
