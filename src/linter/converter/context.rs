use crate::common::{CaseInsensitiveString, QError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::types::{
    DimName, ElementType, Expression, HasTypeDefinition, Members, TypeDefinition, UserDefinedName,
    UserDefinedTypes,
};
use crate::parser;
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

// ========================================================
// ResolvedTypeDefinitions
// ========================================================

#[derive(Debug, PartialEq)]
enum ResolvedTypeDefinitions {
    /// DIM X, DIM X$, X = 42, etc
    Compact(HashSet<TypeQualifier>),
    /// CONST X = 42
    Constant(TypeQualifier),
    /// DIM X AS STRING * 5
    String(u16),
    /// DIM X AS INTEGER
    ExtendedBuiltIn(TypeQualifier),
    /// DIM X AS Card
    UserDefined(BareName),
}

impl ResolvedTypeDefinitions {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            _ => false,
        }
    }

    pub fn is_extended(&self) -> bool {
        match self {
            Self::String(_) | Self::ExtendedBuiltIn(_) | Self::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn has_compact(&self, q: TypeQualifier) -> bool {
        match self {
            Self::Compact(qualifiers) => qualifiers.contains(&q),
            _ => false,
        }
    }

    pub fn add_compact(&mut self, q: TypeQualifier) {
        match self {
            Self::Compact(qualifiers) => {
                if !qualifiers.insert(q) {
                    panic!("Duplicate compact qualifier");
                }
            }
            _ => panic!("Cannot add compact to this set"),
        }
    }
}

//
// SubProgram Type
//

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubProgramType {
    Function,
    Sub,
}

//
// LinterContext
//

#[derive(Debug)]
pub struct LinterContext<'a> {
    parent: Option<Box<LinterContext<'a>>>,
    sub_program: Option<(BareName, SubProgramType)>,
    names: HashMap<BareName, ResolvedTypeDefinitions>,
    user_defined_types: &'a UserDefinedTypes,
    const_values: HashMap<BareName, Variant>,

    /// Collects names of variables and parameters whose type is a user defined type.
    /// These names cannot exist elsewhere as a prefix of a dotted variable, constant, parameter, function or sub name,
    /// regardless of the scope.
    ///
    /// This hash set exists only on the parent level.
    pub names_without_dot: Option<HashSet<BareName>>,
}

impl<'a> LinterContext<'a> {
    pub fn new(user_defined_types: &'a UserDefinedTypes) -> Self {
        Self {
            parent: None,
            sub_program: None,
            names: HashMap::new(),
            user_defined_types,
            names_without_dot: Some(HashSet::new()),
            const_values: HashMap::new(),
        }
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
                ResolvedTypeDefinitions::UserDefined(type_name) => Some(type_name),
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
                if v.is_empty() {
                    Ok(Some(DimName::UserDefined(UserDefinedName {
                        name: first.clone(),
                        type_name: type_name.clone(),
                    })))
                } else {
                    Ok(Some(DimName::Many(
                        UserDefinedName {
                            name: first.clone(),
                            type_name: type_name.clone(),
                        },
                        self.resolve_members(type_name, &v[..])?,
                    )))
                }
            }
            None => {
                // No user defined variable starts with the first dotted name
                Ok(None)
            }
        }
    }

    fn ensure_not_clashing_with_user_defined_var(&self, name: &BareName) -> Result<(), QError> {
        match name.prefix('.') {
            Some(first) => match self.get_user_defined_type_name(&first) {
                Some(_) => Err(QError::syntax_error("Expected: , or end-of-statement")),
                None => Ok(()),
            },
            _ => Ok(()),
        }
    }

    fn ensure_user_defined_var_not_clashing_with_dotted_vars(
        &self,
        name: &BareName,
    ) -> Result<(), QError> {
        if self.names.is_empty() {
            Ok(())
        } else {
            let prefix: BareName = name.clone() + '.';
            for k in self.names.keys() {
                if k.starts_with(&prefix) {
                    return Err(QError::syntax_error("Expected: , or end-of-statement"));
                }
            }
            Ok(())
        }
    }

    // the bool indicates if it was extended or not.
    // TODO improve the bool
    fn resolve_declared_name<T: TypeResolver>(
        &self,
        declared_name: parser::ParamName,
        resolver: &T,
    ) -> Result<(DimName, bool), QError> {
        match declared_name {
            parser::ParamName::Bare(name) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                let q: TypeQualifier = (&name).resolve_into(resolver);
                Ok((DimName::BuiltIn(name, q), false))
            }
            parser::ParamName::Compact(name, q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((DimName::BuiltIn(name, q), false))
            }
            parser::ParamName::ExtendedBuiltIn(name, q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((DimName::BuiltIn(name, q), true))
            }
            parser::ParamName::UserDefined(name, user_defined_type) => {
                if name.contains('.') {
                    Err(QError::IdentifierCannotIncludePeriod)
                } else if self.user_defined_types.contains_key(&user_defined_type) {
                    self.ensure_user_defined_var_not_clashing_with_dotted_vars(&name)?;
                    Ok((
                        DimName::UserDefined(UserDefinedName {
                            name,
                            type_name: user_defined_type,
                        }),
                        true,
                    ))
                } else {
                    Err(QError::TypeNotDefined)
                }
            }
        }
    }

    pub fn push_param<T: TypeResolver>(
        &mut self,
        declared_name: parser::ParamName,
        resolver: &T,
    ) -> Result<(), QError> {
        let (dim_name, is_extended) =
            self.resolve_declared_name(declared_name, resolver)?;
        let name: &BareName = dim_name.as_ref();
        match self.names.get_mut(&name) {
            Some(resolved_type_definitions) => {
                match resolved_type_definitions {
                    ResolvedTypeDefinitions::Compact(existing_set) => {
                        match dim_name.type_definition() {
                            TypeDefinition::BuiltIn(q) => {
                                if existing_set.contains(&q) || is_extended {
                                    return Err(QError::DuplicateDefinition);
                                } else {
                                    existing_set.insert(q);
                                }
                            }
                            _ => {
                                return Err(QError::DuplicateDefinition);
                            }
                        }
                    }
                    _ => {
                        // anything else cannot be extended
                        return Err(QError::DuplicateDefinition);
                    }
                }
            }
            None => match dim_name.type_definition() {
                TypeDefinition::BuiltIn(q) => {
                    if is_extended {
                        self.names
                            .insert(name.clone(), ResolvedTypeDefinitions::ExtendedBuiltIn(q));
                    } else {
                        let mut s: HashSet<TypeQualifier> = HashSet::new();
                        s.insert(q);
                        self.names
                            .insert(name.clone(), ResolvedTypeDefinitions::Compact(s));
                    }
                }
                TypeDefinition::String(_) => {
                    self.names.insert(
                        name.clone(),
                        ResolvedTypeDefinitions::ExtendedBuiltIn(TypeQualifier::DollarString),
                    );
                }
                TypeDefinition::UserDefined(u) => {
                    self.collect_name_without_dot(name);
                    self.names.insert(
                        name.clone(),
                        ResolvedTypeDefinitions::UserDefined(u.clone()),
                    );
                }
                TypeDefinition::FileHandle => panic!("not possible to declare a DIM of filehandle"),
            },
        }
        Ok(())
    }

    pub fn push_const(&mut self, b: BareName, q: TypeQualifier, v: Variant) {
        if self
            .names
            .insert(b.clone(), ResolvedTypeDefinitions::Constant(q))
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
                self.names.insert(b, ResolvedTypeDefinitions::Compact(s));
            }
        }
    }

    pub fn push_dim_extended(&mut self, b: BareName, q: TypeQualifier) {
        if self
            .names
            .insert(b, ResolvedTypeDefinitions::ExtendedBuiltIn(q))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
    }

    pub fn push_dim_string(&mut self, b: BareName, len: u16) {
        if self
            .names
            .insert(b, ResolvedTypeDefinitions::String(len))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
    }

    pub fn push_dim_user_defined(&mut self, b: BareName, u: BareName) {
        self.collect_name_without_dot(&b);
        if self
            .names
            .insert(b, ResolvedTypeDefinitions::UserDefined(u))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
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
                    ResolvedTypeDefinitions::Constant(_) => {
                        // cannot re-assign a constant
                        Err(QError::DuplicateDefinition)
                    }
                    ResolvedTypeDefinitions::Compact(existing_set) => {
                        // if it's not in the existing set, do not add it implicitly yet (might be a parent constant)
                        match name {
                            Name::Bare(b) => {
                                let qualifier: TypeQualifier = resolver.resolve(b);
                                if existing_set.contains(&qualifier) {
                                    Ok(Some(DimName::BuiltIn(b.clone(), qualifier)))
                                } else {
                                    Ok(None)
                                }
                            }
                            Name::Qualified { bare_name: name, qualifier } => {
                                if existing_set.contains(qualifier) {
                                    Ok(Some(DimName::BuiltIn(name.clone(), *qualifier)))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::ExtendedBuiltIn(q) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => Ok(Some(DimName::BuiltIn(b.clone(), *q))),
                            Name::Qualified { bare_name: name, qualifier } => {
                                if q == qualifier {
                                    Ok(Some(DimName::BuiltIn(name.clone(), *qualifier)))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::String(len) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => Ok(Some(DimName::String(b.clone(), *len))),
                            Name::Qualified { bare_name: name, qualifier } => {
                                if TypeQualifier::DollarString == *qualifier {
                                    Ok(Some(DimName::String(name.clone(), *len)))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::UserDefined(u) => {
                        // only possible if the name is bare
                        match name {
                            Name::Bare(b) => Ok(Some(DimName::UserDefined(UserDefinedName {
                                name: b.clone(),
                                type_name: u.clone(),
                            }))),
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
                ResolvedTypeDefinitions::Compact(existing_set) => match name {
                    Name::Bare(b) => {
                        let qualifier: TypeQualifier = resolver.resolve(b);
                        if existing_set.contains(&qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(DimName::BuiltIn(
                                b.clone(),
                                qualifier,
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                    Name::Qualified { bare_name: name, qualifier } => {
                        if existing_set.contains(qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(DimName::BuiltIn(
                                name.clone(),
                                *qualifier,
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                },
                ResolvedTypeDefinitions::Constant(q) => {
                    if name.is_bare_or_of_type(*q) {
                        Ok(Some(Expression::Constant(QualifiedName::new(
                            bare_name.clone(),
                            *q,
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                ResolvedTypeDefinitions::ExtendedBuiltIn(q) => {
                    if name.is_bare_or_of_type(*q) {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::BuiltIn(
                            bare_name.clone(),
                            *q,
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                ResolvedTypeDefinitions::String(len) => {
                    if name.is_bare_or_of_type(TypeQualifier::DollarString) {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::String(
                            bare_name.clone(),
                            *len,
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                ResolvedTypeDefinitions::UserDefined(u) => {
                    if name.is_bare() {
                        // TODO fix me
                        Ok(Some(Expression::Variable(DimName::UserDefined(
                            UserDefinedName {
                                name: bare_name.clone(),
                                type_name: u.clone(),
                            },
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
                ResolvedTypeDefinitions::Constant(q) => {
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
                ResolvedTypeDefinitions::Constant(q) => {
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

    pub fn resolve_param_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<bool, QError> {
        self.do_resolve_assignment(name, resolver)
            .map(|x| x.is_some())
    }

    fn resolve_missing_name_in_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DimName, QError> {
        let QualifiedName { name, qualifier } = name.resolve_into(resolver);
        match self.names.get_mut(name.as_ref()) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                ResolvedTypeDefinitions::Compact(existing_set) => {
                    if existing_set.contains(&qualifier) {
                        Err(QError::DuplicateDefinition)
                    } else {
                        existing_set.insert(qualifier);
                        Ok(DimName::BuiltIn(name, qualifier))
                    }
                }
                _ => Err(QError::DuplicateDefinition),
            },
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(qualifier);
                self.names
                    .insert(name.clone(), ResolvedTypeDefinitions::Compact(s));
                Ok(DimName::BuiltIn(name, qualifier))
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

    pub fn push_function_context(self, bare_name: &BareName) -> Self {
        let mut result = Self {
            parent: None,
            sub_program: Some((bare_name.clone(), SubProgramType::Function)),
            names: HashMap::new(),
            user_defined_types: &self.user_defined_types,
            names_without_dot: None,
            const_values: HashMap::new(),
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn push_sub_context(self, bare_name: &BareName) -> Self {
        let mut result = Self {
            parent: None,
            sub_program: Some((bare_name.clone(), SubProgramType::Sub)),
            names: HashMap::new(),
            user_defined_types: &self.user_defined_types,
            names_without_dot: None,
            const_values: HashMap::new(),
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn pop_context(self) -> Self {
        *self.parent.expect("Stack underflow!")
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

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.sub_program {
            Some((sub_program_name, sub_program_type)) => {
                sub_program_name == name.as_ref() && *sub_program_type == SubProgramType::Function
            }
            None => false,
        }
    }

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

impl<'a> ConstValueResolver for LinterContext<'a> {
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
