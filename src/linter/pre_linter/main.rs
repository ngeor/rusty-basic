use self::sub_program::*;
use crate::common::*;
use crate::linter::const_value_resolver::{ConstLookup, ConstValueResolver};
use crate::linter::pre_linter::PreLinterResult;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::*;
use crate::variant::Variant;
use std::collections::HashMap;

pub fn pre_lint_program(program: &ProgramNode) -> Result<PreLinterResult, QErrorNode> {
    let mut resolver: TypeResolverImpl = TypeResolverImpl::new();
    let mut user_defined_types: UserDefinedTypes = UserDefinedTypes::new();
    let mut function_context = FunctionContext::new();
    let mut sub_context = SubContext::new();
    let mut global_constants: HashMap<BareName, Variant> = HashMap::new();

    for Locatable { element, pos } in program {
        match element {
            TopLevelToken::DefType(def_type) => {
                on_def_type(def_type, &mut resolver);
            }
            TopLevelToken::FunctionDeclaration(name_node, params) => {
                on_function_declaration(
                    name_node,
                    params,
                    *pos,
                    &mut function_context,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::SubDeclaration(bare_name_node, params) => {
                on_sub_declaration(
                    bare_name_node,
                    params,
                    *pos,
                    &mut sub_context,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name: name_node,
                params,
                ..
            }) => {
                on_function_implementation(
                    name_node,
                    params,
                    *pos,
                    &mut function_context,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::SubImplementation(SubImplementation {
                name: bare_name_node,
                params,
                ..
            }) => {
                on_sub_implementation(
                    bare_name_node,
                    params,
                    *pos,
                    &mut sub_context,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::Statement(s) => match s {
                Statement::Const(name_node, expression_node) => {
                    global_const(&mut global_constants, name_node, expression_node)?;
                }
                _ => {}
            },
            TopLevelToken::UserDefinedType(u) => {
                user_defined_type(&mut user_defined_types, &global_constants, u, *pos)
                    .patch_err_pos(*pos)?;
            }
        }
    }

    function_context.post_visit()?;
    sub_context.post_visit()?;

    Ok(PreLinterResult::new(
        function_context.implementations(),
        sub_context.implementations(),
        user_defined_types,
    ))
}

fn on_def_type(def_type: &DefType, resolver: &mut TypeResolverImpl) {
    resolver.set(def_type);
}

fn on_function_declaration(
    name_node: &NameNode,
    params: &ParamNameNodes,
    pos: Location,
    function_context: &mut FunctionContext,
    resolver: &impl TypeResolver,
    user_defined_types: &UserDefinedTypes,
) -> Result<(), QErrorNode> {
    function_context.add_declaration(
        name_node,
        params,
        pos,
        &ContextImpl::new(resolver, user_defined_types),
    )
}

fn on_sub_declaration(
    bare_name_node: &BareNameNode,
    params: &ParamNameNodes,
    pos: Location,
    sub_context: &mut SubContext,
    resolver: &impl TypeResolver,
    user_defined_types: &UserDefinedTypes,
) -> Result<(), QErrorNode> {
    sub_context.add_declaration(
        bare_name_node,
        params,
        pos,
        &ContextImpl::new(resolver, user_defined_types),
    )
}

fn on_function_implementation(
    name_node: &NameNode,
    params: &ParamNameNodes,
    pos: Location,
    function_context: &mut FunctionContext,
    resolver: &impl TypeResolver,
    user_defined_types: &UserDefinedTypes,
) -> Result<(), QErrorNode> {
    function_context.add_implementation(
        name_node,
        params,
        pos,
        &ContextImpl::new(resolver, user_defined_types),
    )
}

fn on_sub_implementation(
    bare_name_node: &BareNameNode,
    params: &ParamNameNodes,
    pos: Location,
    sub_context: &mut SubContext,
    resolver: &impl TypeResolver,
    user_defined_types: &UserDefinedTypes,
) -> Result<(), QErrorNode> {
    sub_context.add_implementation(
        bare_name_node,
        params,
        pos,
        &ContextImpl::new(resolver, user_defined_types),
    )
}

struct ContextImpl<'a, 'b, R> {
    resolver: &'a R,
    user_defined_types: &'b UserDefinedTypes,
}

impl<'a, 'b, R> ContextImpl<'a, 'b, R> {
    pub fn new(resolver: &'a R, user_defined_types: &'b UserDefinedTypes) -> Self {
        Self {
            resolver,
            user_defined_types,
        }
    }
}

impl<'a, 'b, R> TypeResolver for ContextImpl<'a, 'b, R>
where
    R: TypeResolver,
{
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
    }
}

impl<'a, 'b, R> Context for ContextImpl<'a, 'b, R>
where
    R: TypeResolver,
{
    fn is_user_defined_type(&self, name: &BareName) -> bool {
        self.user_defined_types.contains_key(name)
    }
}

// ========================================================
// global constants
// ========================================================

fn global_const(
    global_constants: &mut HashMap<CaseInsensitiveString, Variant>,
    name_node: &NameNode,
    expression_node: &ExpressionNode,
) -> Result<(), QErrorNode> {
    let Locatable { element: name, pos } = name_node;
    let bare_name: &BareName = name.bare_name();
    (match global_constants.get(bare_name) {
        Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
        _ => Ok(()),
    })
    .and_then(|_| global_constants.resolve_const(expression_node))
    .and_then(|v| match name {
        Name::Bare(_) => Ok(v),
        Name::Qualified(_, qualifier) => v.cast(*qualifier).with_err_at(expression_node),
    })
    .map(|casted| {
        global_constants.insert(bare_name.clone(), casted);
        ()
    })
}

impl ConstLookup for HashMap<CaseInsensitiveString, Variant> {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.get(name)
    }
}

// ========================================================
// user defined types
// ========================================================

fn user_defined_type(
    user_defined_types: &mut UserDefinedTypes,
    global_constants: &HashMap<BareName, Variant>,
    user_defined_type: &UserDefinedType,
    pos: Location,
) -> Result<(), QErrorNode> {
    let type_name: &BareName = user_defined_type.as_ref();
    if user_defined_types.contains_key(type_name) {
        // duplicate type definition
        Err(QError::DuplicateDefinition).with_err_no_pos()
    } else {
        let mut resolved_elements: HashMap<BareName, ElementType> = HashMap::new();
        for Locatable {
            element:
                Element {
                    name: element_name,
                    element_type,
                    ..
                },
            pos,
        } in user_defined_type.elements()
        {
            if resolved_elements.contains_key(element_name) {
                // duplicate element name within type
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let resolved_element_type = match element_type {
                ElementType::Integer => ElementType::Integer,
                ElementType::Long => ElementType::Long,
                ElementType::Single => ElementType::Single,
                ElementType::Double => ElementType::Double,
                ElementType::FixedLengthString(str_len_expression_node, _) => {
                    let l: u16 =
                        validate_element_type_str_len(global_constants, str_len_expression_node)?;
                    ElementType::FixedLengthString(
                        Expression::IntegerLiteral(l as i32).at(str_len_expression_node),
                        l,
                    )
                }
                ElementType::UserDefined(Locatable {
                    element: referred_name,
                    pos,
                }) => {
                    if !user_defined_types.contains_key(referred_name) {
                        return Err(QError::TypeNotDefined).with_err_at(pos);
                    }
                    ElementType::UserDefined(referred_name.clone().at(pos))
                }
            };
            resolved_elements.insert(element_name.clone(), resolved_element_type);
        }
        let mut elements: Vec<ElementNode> = vec![];
        for Locatable {
            element: Element { name, .. },
            pos,
        } in user_defined_type.elements()
        {
            let converted_element_type = resolved_elements.remove(name).unwrap();
            elements.push(Element::new(name.clone(), converted_element_type, vec![]).at(pos));
        }
        user_defined_types.insert(
            type_name.clone(),
            UserDefinedType::new(type_name.clone().at(pos), vec![], elements),
        );
        Ok(())
    }
}

fn validate_element_type_str_len(
    global_constants: &HashMap<BareName, Variant>,
    str_len_expression_node: &ExpressionNode,
) -> Result<u16, QErrorNode> {
    let Locatable {
        element: str_len_expression,
        pos,
    } = str_len_expression_node;
    match str_len_expression {
        Expression::IntegerLiteral(i) => {
            // parser already covers that i is between 1..MAX_INT
            Ok(*i as u16)
        }
        Expression::Variable(name_expr, _) => {
            // only constants allowed
            if let Some(qualifier) = name_expr.qualifier() {
                match global_constants.get(name_expr.bare_name()) {
                    // constant exists
                    Some(const_value) => {
                        match const_value {
                            Variant::VInteger(i) => {
                                if qualifier == TypeQualifier::PercentInteger
                                    && *i >= 1
                                    && *i <= crate::variant::MAX_INTEGER
                                {
                                    Ok(*i as u16)
                                } else {
                                    // illegal string length or using wrong qualifier to reference the int constant
                                    Err(QError::InvalidConstant).with_err_at(pos)
                                }
                            }
                            _ => {
                                // only integer constants allowed
                                Err(QError::InvalidConstant).with_err_at(pos)
                            }
                        }
                    }
                    // constant does not exist
                    None => Err(QError::InvalidConstant).with_err_at(pos),
                }
            } else {
                // bare name constant
                match global_constants.get(name_expr.bare_name()) {
                    // constant exists
                    Some(const_value) => {
                        match const_value {
                            Variant::VInteger(i) => {
                                if *i >= 1 && *i <= crate::variant::MAX_INTEGER {
                                    Ok(*i as u16)
                                } else {
                                    // illegal string length
                                    Err(QError::InvalidConstant).with_err_at(pos)
                                }
                            }
                            _ => {
                                // only integer constants allowed
                                Err(QError::InvalidConstant).with_err_at(pos)
                            }
                        }
                    }
                    // constant does not exist
                    None => Err(QError::InvalidConstant).with_err_at(pos),
                }
            }
        }
        _ => panic!("Unexpected string length {:?}", str_len_expression),
    }
}

mod sub_program {
    use crate::built_ins::{BuiltInFunction, BuiltInSub};
    use crate::common::*;
    use crate::linter::type_resolver::{IntoTypeQualifier, TypeResolver};
    use crate::linter::{FunctionSignature, ParamTypes, SubSignature};
    use crate::parser::{BareName, Name, ParamNameNodes};
    use std::collections::HashMap;

    pub trait Context: TypeResolver {
        fn is_user_defined_type(&self, name: &BareName) -> bool;
    }

    pub struct SubprogramContext<T> {
        declarations: HashMap<CaseInsensitiveString, Locatable<T>>,
        implementations: HashMap<CaseInsensitiveString, Locatable<T>>,
    }

    pub type FunctionContext = SubprogramContext<FunctionSignature>;
    pub type SubContext = SubprogramContext<SubSignature>;

    trait CheckSignature<T>
    where
        T: PartialEq,
    {
        /// Checks the signature of the given subprogram name against already known definitions.
        /// Returns an error if the signature doesn't match.
        /// Returns true if the definition already exists.
        /// Returns false if the definition doesn't exist.
        fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, QError>;
    }

    impl<T> CheckSignature<T> for HashMap<CaseInsensitiveString, Locatable<T>>
    where
        T: PartialEq,
    {
        fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, QError> {
            if let Some(Locatable { element, .. }) = self.get(name) {
                if element != signature {
                    Err(QError::TypeMismatch)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }
    }

    pub trait SubprogramName {
        type Signature;

        fn bare_name(&self) -> &BareName;

        fn to_signature(
            &self,
            resolver: &impl TypeResolver,
            qualified_params: ParamTypes,
        ) -> Self::Signature;
    }

    impl SubprogramName for BareName {
        type Signature = SubSignature;

        fn bare_name(&self) -> &BareName {
            self
        }

        fn to_signature(
            &self,
            _resolver: &impl TypeResolver,
            qualified_params: ParamTypes,
        ) -> Self::Signature {
            qualified_params
        }
    }

    impl SubprogramName for Name {
        type Signature = FunctionSignature;

        fn bare_name(&self) -> &BareName {
            self.bare_name()
        }

        fn to_signature(
            &self,
            resolver: &impl TypeResolver,
            qualified_params: ParamTypes,
        ) -> Self::Signature {
            let q = self.qualify(resolver);
            (q, qualified_params)
        }
    }

    impl<T> SubprogramContext<T>
    where
        T: PartialEq,
    {
        pub fn new() -> Self {
            Self {
                declarations: HashMap::new(),
                implementations: HashMap::new(),
            }
        }

        pub fn add_declaration<N: SubprogramName<Signature = T>>(
            &mut self,
            name_node: &Locatable<N>,
            param_name_nodes: &ParamNameNodes,
            declaration_pos: Location,
            context: &impl Context,
        ) -> Result<(), QErrorNode> {
            let Locatable { element: name, pos } = name_node;
            // name does not have to be unique (duplicate identical declarations okay)
            // conflicting declarations to previous declaration or implementation not okay
            let param_types: ParamTypes = params::resolve_param_types(context, param_name_nodes)?;
            let bare_name: &BareName = name.bare_name();
            let signature = name.to_signature(context, param_types);
            self.implementations
                .check_signature(bare_name, &signature)
                .with_err_at(pos)?;
            if !self
                .declarations
                .check_signature(bare_name, &signature)
                .with_err_at(pos)?
            {
                self.declarations
                    .insert(bare_name.clone(), signature.at(declaration_pos));
            }
            Ok(())
        }

        pub fn add_implementation<N: SubprogramName<Signature = T>>(
            &mut self,
            name_node: &Locatable<N>,
            param_name_nodes: &ParamNameNodes,
            implementation_pos: Location,
            context: &impl Context,
        ) -> Result<(), QErrorNode> {
            let Locatable { element: name, pos } = name_node;

            // type must match declaration
            // param count must match declaration
            // param types must match declaration
            // name needs to be unique
            let param_types: ParamTypes = params::resolve_param_types(context, param_name_nodes)?;
            let bare_name: &BareName = name.bare_name();
            let signature = name.to_signature(context, param_types);
            match self.implementations.get(bare_name) {
                Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
                None => {
                    self.declarations
                        .check_signature(bare_name, &signature)
                        .with_err_at(pos)?;
                    self.implementations
                        .insert(bare_name.clone(), signature.at(implementation_pos));
                    Ok(())
                }
            }
        }

        pub fn implementations(self) -> HashMap<CaseInsensitiveString, Locatable<T>> {
            self.implementations
        }

        fn ensure_declarations_are_implemented(&self) -> Result<(), QErrorNode> {
            for (k, v) in self.declarations.iter() {
                if !self.implementations.contains_key(k) {
                    return Err(QError::SubprogramNotDefined).with_err_at(v);
                }
            }
            Ok(())
        }

        fn ensure_does_not_clash_with_built_in<F>(&self, is_built_in: F) -> Result<(), QErrorNode>
        where
            F: Fn(&BareName) -> bool,
        {
            for (k, v) in self.implementations.iter() {
                if is_built_in(k) {
                    return Err(QError::DuplicateDefinition).with_err_at(v);
                }
            }

            Ok(())
        }
    }

    pub trait PostVisit {
        fn post_visit(&self) -> Result<(), QErrorNode>;
    }

    impl PostVisit for FunctionContext {
        fn post_visit(&self) -> Result<(), QErrorNode> {
            self.ensure_declarations_are_implemented()?;
            self.ensure_does_not_clash_with_built_in(|name| {
                Option::<BuiltInFunction>::from(name).is_some()
            })
        }
    }

    impl PostVisit for SubContext {
        fn post_visit(&self) -> Result<(), QErrorNode> {
            // not checking if declarations are present, because in MONEY.BAS there
            // are two SUBs declared but not implemented (and not called either)
            self.ensure_does_not_clash_with_built_in(|name| {
                BuiltInSub::parse_non_keyword_sub(name.as_ref()).is_some()
            })
        }
    }

    mod params {
        //! resolving function/sub parameters

        use super::Context;
        use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
        use crate::linter::type_resolver::IntoTypeQualifier;
        use crate::parser::{
            BareName, BuiltInStyle, ParamName, ParamNameNode, ParamNameNodes, ParamType,
        };

        pub fn resolve_param_types(
            context: &impl Context,
            params: &ParamNameNodes,
        ) -> Result<Vec<ParamType>, QErrorNode> {
            params.iter().map(|p| parameter(context, p)).collect()
        }

        /// Resolves bare parameter types to qualified and ensures user defined types exist.
        fn parameter(
            context: &impl Context,
            param: &ParamNameNode,
        ) -> Result<ParamType, QErrorNode> {
            let Locatable {
                element: param,
                pos,
            } = param;
            let bare_name: &BareName = param.bare_name();
            match &param.var_type {
                ParamType::Bare => {
                    let q = bare_name.qualify(context);
                    Ok(ParamType::BuiltIn(q, BuiltInStyle::Compact))
                }
                ParamType::BuiltIn(q, built_in_style) => {
                    Ok(ParamType::BuiltIn(*q, *built_in_style))
                }
                ParamType::UserDefined(u) => {
                    let type_name: &BareName = u.as_ref();
                    if context.is_user_defined_type(type_name) {
                        Ok(ParamType::UserDefined(u.clone()))
                    } else {
                        Err(QError::TypeNotDefined).with_err_at(pos)
                    }
                }
                ParamType::Array(element_type) => {
                    let dummy_element_param =
                        ParamName::new(bare_name.clone(), element_type.as_ref().clone()).at(pos);
                    let element_param_type = parameter(context, &dummy_element_param)?;
                    Ok(ParamType::Array(Box::new(element_param_type)))
                }
            }
        }
    }
}
