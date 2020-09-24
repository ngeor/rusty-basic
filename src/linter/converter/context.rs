use crate::common::{CaseInsensitiveString, QError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::bare_name_types::BareNameTypes;
use crate::linter::converter::sub_program_type::SubProgramType;
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::types::{
    DimName, DimType, ElementType, Expression, Members, UserDefinedName, UserDefinedTypes,
};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::{HashMap, HashSet};

/*

Naming rules

1. It is possible to have multiple compact variables

e.g. A = 3.14 (resolves as A! by the default rules), A$ = "hello", A% = 1

2. A constant can be referenced either bare or by its correct qualifier

2b. A constant cannot co-exist with other symbols of the same name

3. A bare constant gets its qualifier from the expression and not from the type resolver

4. A constant in a subprogram can override a global constant

5. An extended variable can be referenced either bare or by its correct qualifier
5b. An extended variable cannot co-exist with other symbols of the same name
*/

#[derive(Debug)]
pub struct Context<'a> {
    names: HashMap<BareName, BareNameTypes>,
    const_values: HashMap<BareName, Variant>,
    parent: Option<Box<Context<'a>>>,
    sub_program: Option<(BareName, SubProgramType)>,
    user_defined_types: &'a UserDefinedTypes,
    param_names: HashSet<Name>,

    /// Collects names of variables and parameters whose type is a user defined type.
    /// These names cannot exist elsewhere as a prefix of a dotted variable, constant, parameter, function or sub name,
    /// regardless of the scope.
    ///
    /// This hash set exists only on the parent level.
    pub names_without_dot: Option<HashSet<BareName>>,
    // TODO add TypeResolver reference instead of passing it as parameter
}

impl<'a> Context<'a> {
    pub fn new(user_defined_types: &'a UserDefinedTypes) -> Self {
        Self {
            parent: None,
            sub_program: None,
            names: HashMap::new(),
            user_defined_types,
            names_without_dot: Some(HashSet::new()),
            const_values: HashMap::new(),
            param_names: HashSet::new(),
        }
    }

    fn is_root_context(&self) -> bool {
        self.parent.is_none()
    }

    fn demand_root_context(&self) {
        if !self.is_root_context() {
            panic!("Expected root context");
        }
    }

    pub fn push_function_context(self, bare_name: &BareName) -> Self {
        self.demand_root_context();
        let mut result = Self {
            parent: None,
            sub_program: Some((bare_name.clone(), SubProgramType::Function)),
            names: HashMap::new(),
            user_defined_types: &self.user_defined_types,
            names_without_dot: None,
            const_values: HashMap::new(),
            param_names: HashSet::new(),
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn push_sub_context(self, bare_name: &BareName) -> Self {
        self.demand_root_context();
        let mut result = Self {
            parent: None,
            sub_program: Some((bare_name.clone(), SubProgramType::Sub)),
            names: HashMap::new(),
            user_defined_types: &self.user_defined_types,
            names_without_dot: None,
            const_values: HashMap::new(),
            param_names: HashSet::new(),
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn pop_context(self) -> Self {
        *self.parent.expect("Stack underflow!")
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.names.contains_key(bare_name.as_ref())
    }

    pub fn contains_const(&self, name: &BareName) -> bool {
        match self.names.get(name) {
            Some(resolved_type_definitions) => resolved_type_definitions.is_constant(),
            None => false,
        }
    }

    pub fn contains_compact(&self, name: &BareName, q: TypeQualifier) -> bool {
        match self.names.get(name) {
            Some(resolved_type_definitions) => resolved_type_definitions.has_compact(q),
            None => false,
        }
    }

    pub fn contains_extended(&self, name: &BareName) -> bool {
        match self.names.get(name) {
            Some(resolved_type_definitions) => resolved_type_definitions.is_extended(),
            None => false,
        }
    }

    fn collect_name_without_dot(&mut self, name: &BareName) {
        match &mut self.parent {
            Some(x) => x.collect_name_without_dot(name),
            None => {
                self.names_without_dot
                    .as_mut()
                    .unwrap()
                    .insert(name.clone());
            }
        }
    }

    fn get_user_defined_type_name(&self, name: &BareName) -> Option<&BareName> {
        match self.names.get(name) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                BareNameTypes::UserDefined(type_name) => Some(type_name),
                _ => None,
            },
            None => None,
        }
    }

    fn resolve_members(&self, type_name: &BareName, names: &[BareName]) -> Result<Members, QError> {
        let (first, rest) = names.split_first().expect("Empty names!");
        let user_defined_type = self
            .user_defined_types
            .get(type_name)
            .expect("Type not found!");
        match user_defined_type.find_element(first) {
            Some(element_type) => match element_type {
                ElementType::Integer
                | ElementType::Long
                | ElementType::Single
                | ElementType::Double
                | ElementType::String(_) => {
                    if rest.is_empty() {
                        Ok(Members::Leaf {
                            name: first.clone(),
                            element_type: element_type.clone(),
                        })
                    } else {
                        Err(QError::syntax_error("Cannot navigate after built-in type"))
                    }
                }
                ElementType::UserDefined(u) => {
                    if rest.is_empty() {
                        Ok(Members::Leaf {
                            name: first.clone(),
                            element_type: element_type.clone(),
                        })
                    } else {
                        Ok(Members::Node(
                            UserDefinedName {
                                name: first.clone(),
                                type_name: u.clone(),
                            },
                            Box::new(self.resolve_members(u, rest)?),
                        ))
                    }
                }
            },
            None => Err(QError::ElementNotDefined),
        }
    }

    fn resolve_member(&self, name: &BareName) -> Result<Option<DimName>, QError> {
        let s: String = name.clone().into();
        let mut v: Vec<BareName> = s.split('.').map(|s| s.into()).collect();
        let first: BareName = v.remove(0);
        match self.get_user_defined_type_name(&first) {
            Some(type_name) => {
                let dim_type = if v.is_empty() {
                    DimType::UserDefined(type_name.clone())
                } else {
                    DimType::Many(type_name.clone(), self.resolve_members(type_name, &v[..])?)
                };
                Ok(Some(DimName::new(first.clone(), dim_type)))
            }
            None => {
                // No user defined variable starts with the first dotted name
                Ok(None)
            }
        }
    }

    pub fn push_const(&mut self, b: BareName, q: TypeQualifier, v: Variant) {
        if self
            .names
            .insert(b.clone(), BareNameTypes::Constant(q))
            .is_some()
        {
            panic!("Duplicate definition");
        }

        if self.const_values.insert(b, v).is_some() {
            panic!("Duplicate definition");
        }
    }

    pub fn push_dim_compact(&mut self, b: BareName, q: TypeQualifier) {
        match self.names.get_mut(&b) {
            Some(resolved_type_definitions) => {
                resolved_type_definitions.add_compact(q);
            }
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(q);
                self.names.insert(b, BareNameTypes::Compact(s));
            }
        }
    }

    pub fn push_dim_extended(&mut self, b: BareName, q: TypeQualifier) {
        if self.names.insert(b, BareNameTypes::Extended(q)).is_some() {
            panic!("Duplicate definition!");
        }
    }

    pub fn push_dim_string(&mut self, b: BareName, len: u16) {
        if self
            .names
            .insert(b, BareNameTypes::FixedLengthString(len))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
    }

    pub fn push_dim_user_defined(&mut self, b: BareName, u: BareName) {
        self.collect_name_without_dot(&b);
        if self
            .names
            .insert(b, BareNameTypes::UserDefined(u))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
    }

    pub fn push_compact_param(&mut self, qualified_name: QualifiedName) {
        self.param_names.insert(qualified_name.into());
    }

    pub fn push_extended_param(&mut self, bare_name: BareName) {
        self.param_names.insert(bare_name.into());
    }

    fn do_resolve_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<DimName>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(resolved_type_definitions) => {
                match resolved_type_definitions {
                    BareNameTypes::Constant(_) => {
                        // cannot re-assign a constant
                        Err(QError::DuplicateDefinition)
                    }
                    BareNameTypes::Compact(existing_set) => {
                        // if it's not in the existing set, do not add it implicitly yet (might be a parent constant)
                        match name {
                            Name::Bare(b) => {
                                let qualifier: TypeQualifier = resolver.resolve(b);
                                if existing_set.contains(&qualifier) {
                                    Ok(Some(DimName::new(b.clone(), DimType::BuiltIn(qualifier))))
                                } else {
                                    Ok(None)
                                }
                            }
                            Name::Qualified {
                                bare_name: name,
                                qualifier,
                            } => {
                                if existing_set.contains(qualifier) {
                                    Ok(Some(DimName::new(
                                        name.clone(),
                                        DimType::BuiltIn(*qualifier),
                                    )))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                    BareNameTypes::Extended(q) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => {
                                Ok(Some(DimName::new(b.clone(), DimType::BuiltIn(*q))))
                            }
                            Name::Qualified {
                                bare_name: name,
                                qualifier,
                            } => {
                                if q == qualifier {
                                    Ok(Some(DimName::new(
                                        name.clone(),
                                        DimType::BuiltIn(*qualifier),
                                    )))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    BareNameTypes::FixedLengthString(len) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => Ok(Some(DimName::new(
                                b.clone(),
                                DimType::FixedLengthString(*len),
                            ))),
                            Name::Qualified {
                                bare_name: name,
                                qualifier,
                            } => {
                                if TypeQualifier::DollarString == *qualifier {
                                    Ok(Some(DimName::new(
                                        name.clone(),
                                        DimType::FixedLengthString(*len),
                                    )))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    BareNameTypes::UserDefined(u) => {
                        // only possible if the name is bare
                        match name {
                            Name::Bare(b) => Ok(Some(DimName::new(
                                b.clone(),
                                DimType::UserDefined(u.clone()),
                            ))),
                            _ => Err(QError::TypeMismatch),
                        }
                    }
                }
            }
            None => Ok(if name.is_bare() {
                self.resolve_member(bare_name)?
            } else {
                None
            }),
        }
    }

    fn do_resolve_expression<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                BareNameTypes::Compact(existing_set) => match name {
                    Name::Bare(b) => {
                        let qualifier: TypeQualifier = resolver.resolve(b);
                        if existing_set.contains(&qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(DimName::new(
                                b.clone(),
                                DimType::BuiltIn(qualifier),
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                    Name::Qualified {
                        bare_name: name,
                        qualifier,
                    } => {
                        if existing_set.contains(qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(DimName::new(
                                name.clone(),
                                DimType::BuiltIn(*qualifier),
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                },
                BareNameTypes::Constant(q) => {
                    if name.is_bare_or_of_type(*q) {
                        Ok(Some(Expression::Constant(QualifiedName::new(
                            bare_name.clone(),
                            *q,
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                BareNameTypes::Extended(q) => {
                    if name.is_bare_or_of_type(*q) {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::new(
                            bare_name.clone(),
                            DimType::BuiltIn(*q),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                BareNameTypes::FixedLengthString(len) => {
                    if name.is_bare_or_of_type(TypeQualifier::DollarString) {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::new(
                            bare_name.clone(),
                            DimType::FixedLengthString(*len),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                BareNameTypes::UserDefined(u) => {
                    if name.is_bare() {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::new(
                            bare_name.clone(),
                            DimType::UserDefined(u.clone()),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
            },
            None => Ok(if name.is_bare() {
                self.resolve_member(bare_name)?
                    .map(|n| Expression::Variable(n))
            } else {
                None
            }),
        }
    }

    fn do_resolve_const_expression(&self, name: &Name) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                BareNameTypes::Constant(q) => {
                    if name.is_bare_or_of_type(*q) {
                        Ok(Some(Expression::Constant(QualifiedName::new(
                            bare_name.clone(),
                            *q,
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                _ => Err(QError::InvalidConstant),
            },
            None => Ok(None),
        }
    }

    fn is_constant(&self, name: &Name) -> Result<bool, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                BareNameTypes::Constant(q) => {
                    if name.is_bare_or_of_type(*q) {
                        Ok(true)
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                _ => Ok(false),
            },
            None => Ok(false),
        }
    }

    fn resolve_missing_name_in_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DimName, QError> {
        let QualifiedName {
            bare_name,
            qualifier,
        } = name.resolve_into(resolver);
        match self.names.get_mut(bare_name.as_ref()) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                BareNameTypes::Compact(existing_set) => {
                    if existing_set.contains(&qualifier) {
                        Err(QError::DuplicateDefinition)
                    } else {
                        existing_set.insert(qualifier);
                        Ok(DimName::new(bare_name, DimType::BuiltIn(qualifier)))
                    }
                }
                _ => Err(QError::DuplicateDefinition),
            },
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(qualifier);
                self.names
                    .insert(bare_name.clone(), BareNameTypes::Compact(s));
                Ok(DimName::new(bare_name, DimType::BuiltIn(qualifier)))
            }
        }
    }

    pub fn resolve_missing_name_in_expression<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Expression, QError> {
        let dim_name = self.resolve_missing_name_in_assignment(name, resolver)?;
        Ok(Expression::Variable(dim_name))
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        n: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, QError> {
        // is it param
        // is it constant
        // is it variable
        // is it parent constant
        // is it a sub program?
        // it's a new implicit variable
        self.do_resolve_expression(n, resolver)
            .and_then(|opt| match opt {
                Some(x) => Ok(Some(x)),
                None => self.resolve_parent_const_expression(n),
            })
    }

    fn resolve_parent_const_expression(&self, n: &Name) -> Result<Option<Expression>, QError> {
        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    fn is_parent_constant(&self, n: &Name) -> Result<bool, QError> {
        match &self.parent {
            Some(p) => p.is_constant(n),
            None => Ok(false),
        }
    }

    pub fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, QError> {
        match self.do_resolve_const_expression(n)? {
            Some(e) => Ok(Some(e)),
            None => self.resolve_parent_const_expression(n),
        }
    }

    pub fn is_function_context(&self, bare_name: &BareName) -> bool {
        match &self.sub_program {
            Some((function_name, SubProgramType::Function)) => function_name == bare_name,
            _ => false,
        }
    }

    pub fn is_param<T: TypeResolver>(&self, name: &Name, resolver: &T) -> bool {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(bare_name_types) => {
                if bare_name_types.is_extended() {
                    // it's an extended, so it will appear as bare inside the param_names set
                    let n: Name = bare_name.clone().into();
                    self.param_names.contains(&n)
                } else {
                    // it's a compact, so it will appear as qualified inside the param_names set
                    match name {
                        Name::Bare(_) => {
                            let n: Name = Name::Qualified {
                                bare_name: bare_name.clone(),
                                qualifier: resolver.resolve(bare_name),
                            };
                            self.param_names.contains(&n)
                        }
                        Name::Qualified { .. } => self.param_names.contains(name),
                    }
                }
            }
            None => false,
        }
    }

    #[deprecated]
    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DimName, QError> {
        match self.do_resolve_assignment(name, resolver)? {
            Some(x) => Ok(x),
            None => {
                // maybe a parent constant?
                if self.is_parent_constant(name)? {
                    Err(QError::DuplicateDefinition)
                } else {
                    // just insert it
                    self.resolve_missing_name_in_assignment(name, resolver)
                }
            }
        }
    }
}

impl<'a> ConstValueResolver for Context<'a> {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        match self.const_values.get(name) {
            Some(v) => Some(v),
            None => match &self.parent {
                Some(p) => p.get_resolved_constant(name),
                None => None,
            },
        }
    }
}
