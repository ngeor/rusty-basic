use crate::common::{
    AtLocation, CaseInsensitiveString, Location, QError, QErrorNode, ToLocatableError,
};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::bare_name_types::BareNameTypes;
use crate::linter::converter::sub_program_type::SubProgramType;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::types::{DimName, DimType, Expression, UserDefinedTypes};
use crate::linter::{ArrayDimensions, ExpressionNode};
use crate::parser::{BareName, Name, QualifiedName, QualifiedNameNode, TypeQualifier};
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
    array_dimensions: HashMap<Name, ArrayDimensions>,
    state: ContextState<'a>,
}

impl<'a> Context<'a> {
    pub fn new(user_defined_types: &'a UserDefinedTypes) -> Self {
        Self {
            names: HashMap::new(),
            const_values: HashMap::new(),
            array_dimensions: HashMap::new(),
            state: ContextState::Root {
                user_defined_types,
                names_without_dot: HashSet::new(),
            },
        }
    }

    pub fn contains_any(&self, bare_name: &BareName) -> bool {
        self.names.contains_key(bare_name)
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

    pub fn is_array(&self, name: &Name) -> bool {
        self.array_dimensions.contains_key(name)
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
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<(DimName, bool), QError> {
        match self.resolve_expression(name, resolver)? {
            Some(Expression::Variable(dim_name)) => Ok((dim_name, false)),
            // cannot re-assign a constant
            Some(Expression::Constant(_)) => Err(QError::DuplicateDefinition),
            None => self
                .resolve_missing_name_in_assignment(name, resolver)
                .map(|qualified_name| (qualified_name.into(), true)),
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
                        Name::Qualified(QualifiedName { qualifier, .. }) => *qualifier,
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
            None => Ok(None),
        }
    }

    pub fn resolve_missing_name_in_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<QualifiedName, QError> {
        let QualifiedName {
            bare_name,
            qualifier,
        } = resolver.resolve_name(name);
        self.push_dim_compact(bare_name.clone(), qualifier);
        Ok(QualifiedName::new(bare_name, qualifier))
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

    pub fn resolve_expression_or_add_implicit_variable<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        match self.resolve_expression(name, resolver).with_err_at(pos)? {
            Some(expr) => Ok((expr.at(pos), vec![])),
            None => {
                let qualified_name = self
                    .resolve_missing_name_in_assignment(name, resolver)
                    .with_err_at(pos)?;
                let implicit_variables = vec![qualified_name.clone().at(pos)];
                Ok((
                    Expression::Variable(qualified_name.into()).at(pos),
                    implicit_variables,
                ))
            }
        }
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
                                    let n: Name = QualifiedName::new(
                                        bare_name.clone(),
                                        resolver.resolve(bare_name),
                                    )
                                    .into();
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

    pub fn names_without_dot(self) -> HashSet<BareName> {
        match self.state {
            ContextState::Root {
                names_without_dot, ..
            } => names_without_dot,
            _ => panic!("Expected root context"),
        }
    }

    pub fn register_array_dimensions(&mut self, declared_name: Name, dimensions: ArrayDimensions) {
        self.array_dimensions.insert(declared_name, dimensions);
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
                array_dimensions: HashMap::new(),
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
                array_dimensions: HashMap::new(),
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
