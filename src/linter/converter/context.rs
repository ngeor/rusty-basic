use crate::common::{CaseInsensitiveString, QError, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::bare_name_types::BareNameTypes;
use crate::linter::converter::sub_program_type::SubProgramType;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    BareName, Expression, ExpressionNode, ExpressionType, FunctionMap, Name, QualifiedName,
    QualifiedNameNode, SubMap, TypeQualifier, UserDefinedTypes,
};
use crate::variant::Variant;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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

struct ExpressionTypes {
    map: HashMap<Name, ExpressionType>,
    resolver: Rc<RefCell<TypeResolverImpl>>,
}

impl ExpressionTypes {
    pub fn new(resolver: Rc<RefCell<TypeResolverImpl>>) -> Self {
        Self {
            map: HashMap::new(),
            resolver,
        }
    }

    pub fn get(&self, name: &Name) -> Option<(&ExpressionType, Option<TypeQualifier>)> {
        // try with the name as-is
        match self.map.get(name) {
            Some(expr_type) => Some((expr_type, None)),
            _ => {
                // if the name is bare, try to qualify with the resolver
                if let Name::Bare(bare_name) = name {
                    let qualifier = self.resolve(bare_name);
                    let qualified_name = Name::new(bare_name.clone(), Some(qualifier));
                    match self.map.get(&qualified_name) {
                        Some(expr_type) => Some((expr_type, Some(qualifier))),
                        None => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn insert(&mut self, resolved_name: Name, expr_type: ExpressionType) {
        self.map.insert(resolved_name, expr_type);
    }
}

impl TypeResolver for ExpressionTypes {
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().resolve_char(ch)
    }
}

pub struct Context<'a> {
    names: HashMap<BareName, BareNameTypes>,
    const_values: HashMap<BareName, Variant>,
    state: ContextState<'a>,
    expression_types: ExpressionTypes,
    resolver: Rc<RefCell<TypeResolverImpl>>,
}

impl<'a> Context<'a> {
    pub fn new(
        user_defined_types: &'a UserDefinedTypes,
        resolver: Rc<RefCell<TypeResolverImpl>>,
    ) -> Self {
        Self {
            names: HashMap::new(),
            const_values: HashMap::new(),
            state: ContextState::Root {
                user_defined_types,
                names_without_dot: HashSet::new(),
            },
            expression_types: ExpressionTypes::new(Rc::clone(&resolver)),
            resolver,
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
        if let Some((ExpressionType::Array(_, _), _)) = self.expression_types.get(name) {
            true
        } else {
            false
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

    pub fn resolve_name_in_assignment(
        &mut self,
        name: &Name,
    ) -> Result<(Name, ExpressionType, /* is missing */ bool), QError> {
        match self.resolve_expression(name)? {
            Some(Expression::Variable(var_name, expression_type)) => {
                Ok((var_name, expression_type, false))
            }
            // cannot re-assign a constant
            Some(Expression::Constant(_)) => Err(QError::DuplicateDefinition),
            None => self.resolve_missing_name_in_assignment(name).map(
                |QualifiedName {
                     bare_name,
                     qualifier,
                 }| {
                    (
                        Name::Qualified(QualifiedName::new(bare_name, qualifier)),
                        ExpressionType::BuiltIn(qualifier),
                        true,
                    )
                },
            ),
            _ => panic!("Unexpected result from resolving name expression"),
        }
    }

    fn do_resolve_expression(&self, name: &Name) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(bare_name_types) => match bare_name_types {
                BareNameTypes::Compact(existing_set) => {
                    let q = match name {
                        Name::Bare(b) => self.resolve(b),
                        Name::Qualified(QualifiedName { qualifier, .. }) => *qualifier,
                    };
                    if existing_set.contains(&q) {
                        Ok(Some(Expression::Variable(
                            name.qualify(q),
                            ExpressionType::BuiltIn(q),
                        )))
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
                        Ok(Some(Expression::Variable(
                            name.qualify(*q),
                            ExpressionType::BuiltIn(*q),
                        )))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                BareNameTypes::FixedLengthString(len) => {
                    if name.is_bare_or_of_type(TypeQualifier::DollarString) {
                        Ok(Some(Expression::Variable(
                            name.qualify(TypeQualifier::DollarString),
                            ExpressionType::FixedLengthString(*len),
                        )))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                BareNameTypes::UserDefined(u) => {
                    if name.is_bare() {
                        Ok(Some(Expression::Variable(
                            bare_name.clone().into(),
                            ExpressionType::UserDefined(u.clone()),
                        )))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
            },
            None => {
                if let Some((expr_type, opt_qualifier)) = self.expression_types.get(name) {
                    let new_name = match opt_qualifier {
                        Some(q) => name.qualify(q),
                        _ => name.clone(),
                    };
                    let result_expr_type = match expr_type {
                        // this is the case where we use the array's name without parenthesis, only allowed as parameter to LBound/UBound
                        ExpressionType::Array(element_type, _) => {
                            ExpressionType::Array(element_type.clone(), false)
                        }
                        _ => expr_type.clone(),
                    };
                    Ok(Some(Expression::Variable(new_name, result_expr_type)))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn resolve_missing_name_in_assignment(
        &mut self,
        name: &Name,
    ) -> Result<QualifiedName, QError> {
        let QualifiedName {
            bare_name,
            qualifier,
        } = self.resolve_name(name);
        self.push_dim_compact(bare_name.clone(), qualifier);
        Ok(QualifiedName::new(bare_name, qualifier))
    }

    pub fn resolve_expression(&self, n: &Name) -> Result<Option<Expression>, QError> {
        // is it param
        // is it constant
        // is it variable
        // is it parent constant
        // is it a sub program?
        // it's a new implicit variable
        self.do_resolve_expression(n).and_then(|opt| match opt {
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

    pub fn is_param(&self, name: &Name) -> bool {
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
                                        self.resolve(bare_name),
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

    pub fn register_array_dimensions(
        &mut self,
        resolved_name: Name,
        element_type: ExpressionType,
        with_parenthesis: bool,
    ) {
        self.expression_types.insert(
            resolved_name,
            ExpressionType::Array(Box::new(element_type), with_parenthesis),
        );
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
            let resolver = Rc::clone(&self.resolver);
            Self {
                names: HashMap::new(),
                const_values: HashMap::new(),
                expression_types: ExpressionTypes::new(Rc::clone(&resolver)),
                state: ContextState::Child {
                    param_names: HashSet::new(),
                    parent: Box::new(self),
                    sub_program_name: bare_name,
                    sub_program_type: SubProgramType::Function,
                },
                resolver: Rc::clone(&resolver),
            }
        }

        pub fn push_sub_context(self, bare_name: BareName) -> Self {
            self.demand_root_context();
            let resolver = Rc::clone(&self.resolver);
            Self {
                names: HashMap::new(),
                const_values: HashMap::new(),
                expression_types: ExpressionTypes::new(Rc::clone(&resolver)),
                state: ContextState::Child {
                    param_names: HashSet::new(),
                    parent: Box::new(self),
                    sub_program_name: bare_name,
                    sub_program_type: SubProgramType::Sub,
                },
                resolver: Rc::clone(&resolver),
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

impl<'a> TypeResolver for Context<'a> {
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().resolve_char(ch)
    }
}

pub struct Context2<'a> {
    pub functions: &'a FunctionMap,
    pub subs: &'a SubMap,
    pub user_defined_types: &'a UserDefinedTypes,
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
}

impl<'a> Context2<'a> {
    pub fn resolve_expression(
        &mut self,
        _expr_node: ExpressionNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        todo!()
    }
}
