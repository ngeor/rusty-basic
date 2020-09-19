use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    AtLocation, CaseInsensitiveString, Locatable, Location, PatchErrPos, QError, QErrorNode,
    ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::types::{
    ElementType, ParamTypeDefinition, ParamTypes, UserDefinedType, UserDefinedTypes,
};
use crate::parser;
use crate::parser::{
    BareName, BareNameNode, DeclaredName, DeclaredNameNode, DeclaredNameNodes, Expression,
    ExpressionNode, Name, NameNode, Operator, ProgramNode, Statement, TopLevelToken, TypeQualifier,
    UnaryOperator,
};
use crate::variant::Variant;
use std::cmp::Ordering;
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
            TopLevelToken::FunctionImplementation(name_node, params, _) => {
                function_context.add_implementation(
                    name_node,
                    params,
                    *pos,
                    &resolver,
                    &user_defined_types,
                )?;
            }
            TopLevelToken::SubImplementation(bare_name_node, params, _) => {
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
                user_defined_type(&mut user_defined_types, &global_constants, u)
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
    let bare_name: &BareName = name.as_ref();
    if global_constants.contains_key(bare_name) {
        return Err(QError::DuplicateDefinition).with_err_at(pos);
    }
    let Locatable {
        element: expression,
        pos,
    } = expression_node;
    let v: Variant = resolve_const_value(global_constants, expression).patch_err_pos(pos)?;
    match name {
        Name::Bare(b) => {
            global_constants.insert(b.clone(), v);
        }
        Name::Qualified { name, qualifier } => {
            let casted = v.cast(*qualifier).with_err_at(pos)?;
            global_constants.insert(name.clone(), casted);
        }
    }
    Ok(())
}

fn resolve_const_value(
    global_constants: &HashMap<CaseInsensitiveString, Variant>,

    expression: &Expression,
) -> Result<Variant, QErrorNode> {
    match expression {
        Expression::SingleLiteral(f) => Ok(Variant::VSingle(*f)),
        Expression::DoubleLiteral(d) => Ok(Variant::VDouble(*d)),
        Expression::StringLiteral(s) => Ok(Variant::VString(s.clone())),
        Expression::IntegerLiteral(i) => Ok(Variant::VInteger(*i)),
        Expression::LongLiteral(l) => Ok(Variant::VLong(*l)),
        Expression::VariableName(name) => match name {
            Name::Bare(name) => match global_constants.get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(QError::InvalidConstant).with_err_no_pos(),
            },
            Name::Qualified { name, qualifier } => match global_constants.get(name) {
                Some(v) => {
                    let v_q = match v {
                        Variant::VDouble(_) => TypeQualifier::HashDouble,
                        Variant::VSingle(_) => TypeQualifier::BangSingle,
                        Variant::VInteger(_) => TypeQualifier::PercentInteger,
                        Variant::VLong(_) => TypeQualifier::AmpersandLong,
                        Variant::VString(_) => TypeQualifier::DollarString,
                        _ => {
                            panic!("should not have been possible to store a constant of this type")
                        }
                    };
                    if v_q == *qualifier {
                        Ok(v.clone())
                    } else {
                        Err(QError::TypeMismatch).with_err_no_pos()
                    }
                }
                None => Err(QError::InvalidConstant).with_err_no_pos(),
            },
        },
        Expression::FunctionCall(_, _) => Err(QError::InvalidConstant).with_err_no_pos(),
        Expression::BinaryExpression(op, left, right) => {
            let Locatable { element, pos } = left.as_ref();
            let v_left = resolve_const_value(global_constants, element).patch_err_pos(*pos)?;
            let Locatable { element, pos } = right.as_ref();
            let v_right = resolve_const_value(global_constants, element).patch_err_pos(*pos)?;
            match *op {
                Operator::Less => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Less).into())
                }
                Operator::LessOrEqual => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Less || order == Ordering::Equal).into())
                }
                Operator::Equal => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Equal).into())
                }
                Operator::GreaterOrEqual => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Greater || order == Ordering::Equal).into())
                }
                Operator::Greater => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Greater).into())
                }
                Operator::NotEqual => {
                    let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                    Ok((order == Ordering::Less || order == Ordering::Greater).into())
                }
                Operator::Plus => v_left.plus(v_right).with_err_at(*pos),
                Operator::Minus => v_left.minus(v_right).with_err_at(*pos),
                Operator::Multiply => v_left.multiply(v_right).with_err_at(*pos),
                Operator::Divide => v_left.divide(v_right).with_err_at(*pos),
                Operator::And => v_left.and(v_right).with_err_at(*pos),
                Operator::Or => v_left.or(v_right).with_err_at(*pos),
            }
        }
        Expression::UnaryExpression(op, child) => {
            let Locatable { element, pos } = child.as_ref();
            let v = resolve_const_value(global_constants, element).patch_err_pos(*pos)?;
            match *op {
                UnaryOperator::Minus => v.negate().with_err_at(*pos),
                UnaryOperator::Not => v.unary_not().with_err_at(*pos),
            }
        }
        Expression::Parenthesis(child) => {
            let Locatable { element, pos } = child.as_ref();
            resolve_const_value(global_constants, element).patch_err_pos(*pos)
        }
        Expression::FileHandle(_) => Err(QError::InvalidConstant).with_err_no_pos(),
    }
}

// ========================================================
// user defined types
// ========================================================

fn user_defined_type(
    user_defined_types: &mut UserDefinedTypes,
    global_constants: &HashMap<BareName, Variant>,
    user_defined_type: &parser::UserDefinedType,
) -> Result<(), QErrorNode> {
    let type_name: &BareName = user_defined_type.name.as_ref();
    if user_defined_types.contains_key(type_name) {
        // duplicate type definition
        Err(QError::DuplicateDefinition).with_err_no_pos()
    } else {
        let mut resolved_elements: HashMap<BareName, ElementType> = HashMap::new();
        for Locatable { element, pos } in user_defined_type.elements.iter() {
            let element_name: &BareName = &element.name;
            if resolved_elements.contains_key(element_name) {
                // duplicate element name within type
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let resolved_element_type = match &element.element_type {
                parser::ElementType::Integer => ElementType::Integer,
                parser::ElementType::Long => ElementType::Long,
                parser::ElementType::Single => ElementType::Single,
                parser::ElementType::Double => ElementType::Double,
                parser::ElementType::String(str_len_expression_node) => {
                    let l: u16 =
                        validate_element_type_str_len(global_constants, str_len_expression_node)?;
                    ElementType::String(l)
                }
                parser::ElementType::UserDefined(Locatable {
                    element: referred_name,
                    pos,
                }) => {
                    if !user_defined_types.contains_key(referred_name) {
                        return Err(QError::TypeNotDefined).with_err_at(pos);
                    }
                    ElementType::UserDefined(referred_name.clone())
                }
            };
            resolved_elements.insert(element_name.clone(), resolved_element_type);
        }
        user_defined_types.insert(type_name.clone(), UserDefinedType::new(resolved_elements));
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
        Expression::VariableName(v) => {
            // only constants allowed
            match v {
                Name::Bare(b) => {
                    // bare name constant
                    match global_constants.get(b) {
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
                Name::Qualified { name, qualifier } => {
                    match global_constants.get(name) {
                        // constant exists
                        Some(const_value) => {
                            match const_value {
                                Variant::VInteger(i) => {
                                    if *qualifier == TypeQualifier::PercentInteger
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
                }
            }
        }
        _ => panic!("Unexpected string length {:?}", str_len_expression),
    }
}

struct SubProgramContext<T> {
    declarations: HashMap<CaseInsensitiveString, Locatable<T>>,
    implementations: HashMap<CaseInsensitiveString, Locatable<T>>,
}

impl<T> SubProgramContext<T> {
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
        params: &DeclaredNameNodes,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<Vec<ParamTypeDefinition>, QErrorNode> {
        params
            .iter()
            .map(|p| self.parameter(p, resolver, user_defined_types))
            .collect()
    }

    fn parameter(
        &self,
        param: &DeclaredNameNode,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<ParamTypeDefinition, QErrorNode> {
        let Locatable {
            element: declared_name,
            pos,
        } = param;
        let DeclaredName {
            name,
            type_definition,
        } = declared_name;
        match type_definition {
            parser::TypeDefinition::Bare => {
                let q: TypeQualifier = resolver.resolve(name);
                Ok(ParamTypeDefinition::BuiltIn(q))
            }
            parser::TypeDefinition::CompactBuiltIn(q)
            | parser::TypeDefinition::ExtendedBuiltIn(q) => Ok(ParamTypeDefinition::BuiltIn(*q)),
            parser::TypeDefinition::UserDefined(u) => {
                if user_defined_types.contains_key(u) {
                    Ok(ParamTypeDefinition::UserDefined(u.clone()))
                } else {
                    Err(QError::TypeNotDefined).with_err_at(pos)
                }
            }
        }
    }
}

pub type FunctionSignature = (TypeQualifier, ParamTypes);
pub type FunctionSignatureNode = Locatable<FunctionSignature>;
pub type FunctionMap = HashMap<CaseInsensitiveString, FunctionSignatureNode>;
type FunctionContext = SubProgramContext<FunctionSignature>;

impl FunctionContext {
    pub fn add_declaration(
        &mut self,
        name_node: &NameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        let q_name: TypeQualifier = name.resolve_into(resolver);
        let bare_name: &BareName = name.as_ref();
        self.check_implementation_type(bare_name, &q_name, &q_params)?;
        match self.declarations.get(bare_name) {
            Some(_) => self
                .check_declaration_type(bare_name, &q_name, &q_params)
                .with_err_no_pos(),
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
        params: &DeclaredNameNodes,
        implementation_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = name_node;

        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        let q_name: TypeQualifier = name.resolve_into(resolver);
        let bare_name: &BareName = name.as_ref();
        match self.implementations.get(bare_name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_no_pos(),
            None => {
                self.check_declaration_type(bare_name, &q_name, &q_params)
                    .with_err_no_pos()?;
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
    ) -> Result<(), QErrorNode> {
        match self.implementations.get(name) {
            Some(Locatable {
                element: (e_name, e_params),
                ..
            }) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_no_pos();
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

pub type SubSignature = ParamTypes;
pub type SubSignatureNode = Locatable<SubSignature>;
pub type SubMap = HashMap<CaseInsensitiveString, SubSignatureNode>;
type SubContext = SubProgramContext<SubSignature>;

impl SubContext {
    pub fn add_declaration(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = bare_name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        self.check_implementation_type(name, &q_params)
            .with_err_no_pos()?;
        match self.declarations.get(name) {
            Some(_) => self
                .check_declaration_type(name, &q_params)
                .with_err_no_pos(),
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
        params: &DeclaredNameNodes,
        implementation_pos: Location,
        resolver: &TypeResolverImpl,
        user_defined_types: &UserDefinedTypes,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = bare_name_node;
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: ParamTypes = self.parameters(params, resolver, user_defined_types)?;
        match self.implementations.get(name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_no_pos(),
            None => {
                self.check_declaration_type(name, &q_params)
                    .with_err_no_pos()?;
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
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInSub> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v);
            }
        }

        Ok(())
    }
}
