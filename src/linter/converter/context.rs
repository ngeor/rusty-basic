use crate::common::{CaseInsensitiveString, QError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::bare_name_types::BareNameTypes;
use crate::linter::converter::sub_program_type::SubProgramType;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::types::{
    DimName, DimType, ElementType, Expression, Members, UserDefinedName, UserDefinedType,
    UserDefinedTypes,
};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

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
enum ContextState<'a> {
    Root {
        user_defined_types: &'a UserDefinedTypes,

        /// Collects names of variables and parameters whose type is a user defined type.
        /// These names cannot exist elsewhere as a prefix of a dotted variable, constant, parameter, function or sub name,
        /// regardless of the scope.
        ///
        /// This hash set exists only on the parent level.
        names_without_dot: HashSet<BareName>,
    },
    Child {
        parent: Box<Context<'a>>,
        param_names: HashSet<Name>,
        sub_program_name: BareName,
        sub_program_type: SubProgramType,
    },
}

#[derive(Debug)]
pub struct Context<'a> {
    names: HashMap<BareName, BareNameTypes>,
    const_values: HashMap<BareName, Variant>,
    state: ContextState<'a>,
}

impl<'a> Context<'a> {
    pub fn new(user_defined_types: &'a UserDefinedTypes) -> Self {
        Self {
            names: HashMap::new(),
            const_values: HashMap::new(),
            state: ContextState::Root {
                user_defined_types,
                names_without_dot: HashSet::new(),
            },
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.names.contains_key(bare_name.as_ref())
    }

    pub fn contains_const(&self, name: &BareName) -> bool {
        match self.names.get(name) {
            Some(BareNameTypes::Constant(_)) => true,
            _ => false,
        }
    }

    pub fn contains_compact(&self, name: &BareName, q: TypeQualifier) -> bool {
        match self.names.get(name) {
            Some(bare_name_types) => bare_name_types.has_compact(q),
            None => false,
        }
    }

    pub fn contains_extended(&self, name: &BareName) -> bool {
        match self.names.get(name) {
            Some(bare_name_types) => bare_name_types.is_extended(),
            None => false,
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
            Some(bare_name_types) => {
                bare_name_types.add_compact(q);
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
        self.collect_name_without_dot(b.clone());
        if self
            .names
            .insert(b, BareNameTypes::UserDefined(u))
            .is_some()
        {
            panic!("Duplicate definition!");
        }
    }

    fn collect_name_without_dot(&mut self, name: BareName) {
        match &mut self.state {
            ContextState::Child { parent, .. } => parent.collect_name_without_dot(name),
            ContextState::Root {
                names_without_dot, ..
            } => {
                names_without_dot.insert(name);
            }
        }
    }

    pub fn push_compact_param(&mut self, qualified_name: QualifiedName) {
        self.push_param(qualified_name.into());
    }

    pub fn push_extended_param(&mut self, bare_name: BareName) {
        self.push_param(bare_name.into());
    }

    fn push_param(&mut self, name: Name) {
        match &mut self.state {
            ContextState::Child { param_names, .. } => {
                if !param_names.insert(name) {
                    panic!("Duplicate parameter");
                }
            }
            _ => panic!("Root context cannot have parameters"),
        }
    }

    pub fn resolve_name_in_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<DimName>, QError> {
        match self.resolve_expression(name, resolver)? {
            Some(Expression::Variable(dim_name)) => Ok(Some(dim_name)),
            // cannot re-assign a constant
            Some(Expression::Constant(_)) => Err(QError::DuplicateDefinition),
            None => Ok(None),
            _ => panic!("Unexpected result from resolving name expression"),
        }
    }

    fn do_resolve_expression<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(bare_name_types) => match bare_name_types {
                BareNameTypes::Compact(existing_set) => {
                    let q = match name {
                        Name::Bare(b) => resolver.resolve(b),
                        Name::Qualified { qualifier, .. } => *qualifier,
                    };
                    if existing_set.contains(&q) {
                        Ok(Some(Expression::Variable(DimName::new(
                            bare_name.clone(),
                            DimType::BuiltIn(q),
                        ))))
                    } else {
                        Ok(None)
                    }
                }
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
                        Ok(Some(Expression::Variable(DimName::new(
                            bare_name.clone(),
                            DimType::UserDefined(u.clone()),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
            },
            None => {
                Ok(resolve_members::resolve_member(self, name)?.map(|n| Expression::Variable(n)))
            }
        }
    }

    pub fn resolve_missing_name_in_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DimName, QError> {
        let QualifiedName {
            bare_name,
            qualifier,
        } = resolver.resolve_name(name);
        match self.names.get_mut(bare_name.as_ref()) {
            Some(bare_name_types) => match bare_name_types {
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
                None => resolve_const::resolve_parent_const_expression(self, n),
            })
    }

    pub fn is_function_context(&self, bare_name: &BareName) -> bool {
        match &self.state {
            ContextState::Child {
                sub_program_name,
                sub_program_type: SubProgramType::Function,
                ..
            } => sub_program_name == bare_name,
            _ => false,
        }
    }

    pub fn is_param<T: TypeResolver>(&self, name: &Name, resolver: &T) -> bool {
        match &self.state {
            ContextState::Child { param_names, .. } => {
                let bare_name: &BareName = name.as_ref();
                match self.names.get(bare_name) {
                    Some(bare_name_types) => {
                        if bare_name_types.is_extended() {
                            // it's an extended, so it will appear as bare inside the param_names set
                            let n: Name = bare_name.clone().into();
                            param_names.contains(&n)
                        } else {
                            // it's a compact, so it will appear as qualified inside the param_names set
                            match name {
                                Name::Bare(_) => {
                                    let n: Name = Name::Qualified {
                                        bare_name: bare_name.clone(),
                                        qualifier: resolver.resolve(bare_name),
                                    };
                                    param_names.contains(&n)
                                }
                                Name::Qualified { .. } => param_names.contains(name),
                            }
                        }
                    }
                    None => false,
                }
            }
            _ => false,
        }
    }

    pub fn take_names_without_dot(self) -> HashSet<BareName> {
        match self.state {
            ContextState::Root {
                names_without_dot, ..
            } => names_without_dot,
            _ => panic!("Expected root context"),
        }
    }
}

impl<'a> ConstValueResolver for Context<'a> {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        match self.const_values.get(name) {
            Some(v) => Some(v),
            None => match &self.state {
                ContextState::Child { parent, .. } => parent.get_resolved_constant(name),
                _ => None,
            },
        }
    }
}

mod context_management {
    use super::*;

    impl<'a> Context<'a> {
        fn demand_root_context(&self) {
            match &self.state {
                ContextState::Child { .. } => panic!("Expected root context"),
                _ => {}
            }
        }

        pub fn push_function_context(self, bare_name: BareName) -> Self {
            self.demand_root_context();
            Self {
                names: HashMap::new(),
                const_values: HashMap::new(),
                state: ContextState::Child {
                    param_names: HashSet::new(),
                    parent: Box::new(self),
                    sub_program_name: bare_name,
                    sub_program_type: SubProgramType::Function,
                },
            }
        }

        pub fn push_sub_context(self, bare_name: BareName) -> Self {
            self.demand_root_context();
            Self {
                names: HashMap::new(),
                const_values: HashMap::new(),
                state: ContextState::Child {
                    param_names: HashSet::new(),
                    parent: Box::new(self),
                    sub_program_name: bare_name,
                    sub_program_type: SubProgramType::Sub,
                },
            }
        }

        pub fn pop_context(self) -> Self {
            match self.state {
                ContextState::Child { parent, .. } => *parent,
                _ => panic!("Stack underflow!"),
            }
        }
    }
}

mod resolve_members {
    use super::*;

    pub fn resolve_member(context: &Context, name: &Name) -> Result<Option<DimName>, QError> {
        let (bare_name, opt_qualifier) = match name {
            Name::Bare(bare_name) => (bare_name, None),
            Name::Qualified {
                bare_name,
                qualifier,
            } => (bare_name, Some(*qualifier)),
        };
        let s: String = bare_name.clone().into();
        let mut v: Vec<BareName> = s.split('.').map(|s| s.into()).collect();
        let first: BareName = v.remove(0);
        if v.is_empty() {
            return Ok(None);
        }
        match context.names.get(&first) {
            Some(BareNameTypes::UserDefined(type_name)) => {
                let dim_type = DimType::Many(
                    type_name.clone(),
                    resolve_members(&context.state, type_name, &v[..], opt_qualifier)?,
                );
                Ok(Some(DimName::new(first.clone(), dim_type)))
            }
            _ => Ok(None),
        }
    }

    fn resolve_members(
        state: &ContextState,
        type_name: &BareName,
        names: &[BareName],
        opt_qualifier: Option<TypeQualifier>,
    ) -> Result<Members, QError> {
        let (first, rest) = names.split_first().expect("Empty names!");
        let user_defined_type = get_user_defined_type(state, type_name).expect("Type not found!");
        match user_defined_type.find_element(first) {
            Some(element_type) => match element_type {
                ElementType::Integer
                | ElementType::Long
                | ElementType::Single
                | ElementType::Double
                | ElementType::FixedLengthString(_) => {
                    if rest.is_empty() {
                        let is_correct_type = match opt_qualifier {
                            Some(q) => {
                                let element_q: TypeQualifier = element_type.try_into().unwrap();
                                q == element_q
                            }
                            None => true,
                        };
                        if is_correct_type {
                            Ok(Members::Leaf {
                                name: first.clone(),
                                element_type: element_type.clone(),
                            })
                        } else {
                            Err(QError::TypeMismatch)
                        }
                    } else {
                        Err(QError::syntax_error("Cannot navigate after built-in type"))
                    }
                }
                ElementType::UserDefined(u) => {
                    if rest.is_empty() {
                        if opt_qualifier.is_none() {
                            Ok(Members::Leaf {
                                name: first.clone(),
                                element_type: element_type.clone(),
                            })
                        } else {
                            // e.g. c.Address$ where c.Address is nested user defined
                            Err(QError::TypeMismatch)
                        }
                    } else {
                        Ok(Members::Node(
                            UserDefinedName {
                                name: first.clone(),
                                type_name: u.clone(),
                            },
                            Box::new(resolve_members(state, u, rest, opt_qualifier)?),
                        ))
                    }
                }
            },
            None => Err(QError::ElementNotDefined),
        }
    }

    fn get_user_defined_type<'a>(
        state: &'a ContextState,
        type_name: &'a BareName,
    ) -> Option<&'a UserDefinedType> {
        match state {
            ContextState::Child { parent, .. } => get_user_defined_type(&parent.state, type_name),
            ContextState::Root {
                user_defined_types, ..
            } => user_defined_types.get(type_name),
        }
    }
}

mod resolve_const {
    use super::*;

    pub fn resolve_parent_const_expression(
        context: &Context,
        n: &Name,
    ) -> Result<Option<Expression>, QError> {
        match &context.state {
            ContextState::Child { parent, .. } => resolve_const_expression(parent, n),
            _ => Ok(None),
        }
    }

    fn resolve_const_expression(context: &Context, n: &Name) -> Result<Option<Expression>, QError> {
        match do_resolve_const_expression(context, n)? {
            Some(e) => Ok(Some(e)),
            None => resolve_parent_const_expression(context, n),
        }
    }

    fn do_resolve_const_expression(
        context: &Context,
        name: &Name,
    ) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match context.names.get(bare_name) {
            Some(BareNameTypes::Constant(q)) => {
                if name.is_bare_or_of_type(*q) {
                    Ok(Some(Expression::Constant(QualifiedName::new(
                        bare_name.clone(),
                        *q,
                    ))))
                } else {
                    Err(QError::DuplicateDefinition)
                }
            }
            _ => Ok(None),
        }
    }
}
