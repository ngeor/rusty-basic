use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    AtLocation, CaseInsensitiveString, Locatable, Location, PatchErrPos, QError, QErrorNode,
    ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    BareName, BareNameNode, BuiltInStyle, Element, ElementNode, ElementType, Expression,
    ExpressionNode, FunctionImplementation, FunctionMap, FunctionSignature, Name, NameNode,
    ParamName, ParamNameNode, ParamNameNodes, ParamType, ParamTypes, ProgramNode, QualifiedName,
    Statement, SubImplementation, SubMap, SubSignature, TopLevelToken, TypeQualifier,
    UserDefinedType, UserDefinedTypes,
};
use crate::variant::Variant;
use std::collections::HashMap;

pub fn parse_subprograms_and_types(
    program: &ProgramNode,
) -> Result<(FunctionMap, SubMap, UserDefinedTypes), QErrorNode> {
    let mut resolver: TypeResolverImpl = TypeResolverImpl::new();
    let mut user_defined_types: UserDefinedTypes = UserDefinedTypes::new();
    let mut function_context = FunctionContext::new();
    let mut sub_context = SubContext::new();
    let mut global_constants: HashMap<BareName, Variant> = HashMap::new();

    for Locatable { element, pos } in program {
        match element {
            TopLevelToken::DefType(def_type) => {
                resolver.set(def_type);
            }
            TopLevelToken::FunctionDeclaration(name_node, params) => {
                function_context.add_declaration(
                    name_node,
                    params,
                    *pos,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::SubDeclaration(bare_name_node, params) => {
                sub_context.add_declaration(
                    bare_name_node,
                    params,
                    *pos,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name: name_node,
                params,
                ..
            }) => {
                function_context.add_implementation(
                    name_node,
                    params,
                    *pos,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::SubImplementation(SubImplementation {
                name: bare_name_node,
                params,
                ..
            }) => {
                sub_context.add_implementation(
                    bare_name_node,
                    params,
                    *pos,
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

    Ok((
        function_context.implementations,
        sub_context.implementations,
        user_defined_types,
    ))
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
    if global_constants.contains_key(bare_name) {
        return Err(QError::DuplicateDefinition).with_err_at(pos);
    }
    let v: Variant = global_constants.resolve_const_value_node(expression_node)?;
    match name {
        Name::Bare(b) => {
            global_constants.insert(b.clone(), v);
        }
        Name::Qualified(QualifiedName {
            bare_name,
            qualifier,
        }) => {
            let casted = v.cast(*qualifier).with_err_at(expression_node)?;
            global_constants.insert(bare_name.clone(), casted);
        }
    }
    Ok(())
}

impl ConstValueResolver for HashMap<CaseInsensitiveString, Variant> {
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

struct SubprogramContext<T> {
    declarations: HashMap<CaseInsensitiveString, Locatable<T>>,
    implementations: HashMap<CaseInsensitiveString, Locatable<T>>,
}

impl<T> SubprogramContext<T> {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    // ========================================================
    // resolving function/sub parameters
    // ========================================================

    fn parameters(
        &self,
        params: &ParamNameNodes,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<Vec<ParamType>, QErrorNode> {
        params
            .iter()
            .map(|p| self.parameter(p, resolver, user_defined_types))
            .collect()
    }

    fn parameter(
        &self,
        param: &ParamNameNode,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<ParamType, QErrorNode> {
        let Locatable {
            element: param,
            pos,
        } = param;
        let bare_name: &BareName = param.as_ref();
        match &param.param_type {
            ParamType::Bare => {
                let q: TypeQualifier = resolver.resolve(bare_name);
                Ok(ParamType::BuiltIn(q, BuiltInStyle::Compact))
            }
            ParamType::BuiltIn(q, built_in_style) => Ok(ParamType::BuiltIn(*q, *built_in_style)),
            ParamType::UserDefined(u) => {
                let type_name: &BareName = u.as_ref();
                if user_defined_types.contains_key(type_name) {
                    Ok(ParamType::UserDefined(u.clone()))
                } else {
                    Err(QError::TypeNotDefined).with_err_at(pos)
                }
            }
            ParamType::Array(element_type) => {
                let dummy_element_param =
                    ParamName::new(bare_name.clone(), element_type.as_ref().clone()).at(pos);
                let element_param_type =
                    self.parameter(&dummy_element_param, resolver, user_defined_types)?;
                Ok(ParamType::Array(Box::new(element_param_type)))
            }
        }
    }
}

type FunctionContext = SubprogramContext<FunctionSignature>;

impl FunctionContext {
    pub fn add_declaration(
        &mut self,
        name_node: &NameNode,
        params: &ParamNameNodes,
        declaration_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, pos } = name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        let q_name: TypeQualifier = resolver.resolve_name(name).qualifier;
        let bare_name: &BareName = name.bare_name();
        self.check_implementation_type(bare_name, &q_name, &q_params)
            .with_err_at(pos)?;
        match self.declarations.get(bare_name) {
            Some(_) => self
                .check_declaration_type(bare_name, &q_name, &q_params)
                .with_err_at(pos),
            None => {
                self.declarations
                    .insert(bare_name.clone(), (q_name, q_params).at(declaration_pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        name_node: &NameNode,
        params: &ParamNameNodes,
        implementation_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, pos } = name_node;

        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        let q_name: TypeQualifier = resolver.resolve_name(name).qualifier;
        let bare_name: &BareName = name.bare_name();
        match self.implementations.get(bare_name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
            None => {
                self.check_declaration_type(bare_name, &q_name, &q_params)
                    .with_err_at(pos)?;
                self.implementations
                    .insert(bare_name.clone(), (q_name, q_params).at(implementation_pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.declarations.get(name) {
            Some(Locatable {
                element: (e_name, e_params),
                ..
            }) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.implementations.get(name) {
            Some(Locatable {
                element: (e_name, e_params),
                ..
            }) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn post_visit(&self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInFunction> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v);
            }
        }

        Ok(())
    }
}

type SubContext = SubprogramContext<SubSignature>;

impl SubContext {
    pub fn add_declaration(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &ParamNameNodes,
        declaration_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, pos } = bare_name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        self.check_implementation_type(name, &q_params)
            .with_err_at(pos)?;
        match self.declarations.get(name) {
            Some(_) => self
                .check_declaration_type(name, &q_params)
                .with_err_at(pos),
            None => {
                self.declarations
                    .insert(name.clone(), q_params.at(declaration_pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &ParamNameNodes,
        implementation_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, pos } = bare_name_node;
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        match self.implementations.get(name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
            None => {
                self.check_declaration_type(name, &q_params)
                    .with_err_at(pos)?;
                self.implementations
                    .insert(name.clone(), q_params.at(implementation_pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.declarations.get(name) {
            Some(Locatable {
                element: e_params, ..
            }) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.implementations.get(name) {
            Some(Locatable {
                element: e_params, ..
            }) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn post_visit(&self) -> Result<(), QErrorNode> {
        // not checking if declarations are present, because in MONEY.BAS there
        // are two SUBs declared but not implemented (and not called either)

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInSub> = BuiltInSub::parse_non_keyword_sub(k.as_ref());
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v);
            }
        }

        Ok(())
    }
}
