use crate::common::{Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::types::{
    Expression, Members, ResolvedDeclaredName, ResolvedElement, ResolvedElementType,
    ResolvedTypeDefinition, ResolvedUserDefinedTypes, UserDefinedName,
};
use crate::parser::{
    BareName, CanCastTo, DeclaredName, Name, QualifiedName, TypeDefinition, TypeQualifier,
    WithTypeQualifier,
};
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

// ========================================================
// ResolvedTypeDefinitions
// ========================================================

#[derive(Debug, PartialEq)]
enum ResolvedTypeDefinitions {
    /// DIM X, DIM X$, X = 42, etc
    Compact(HashSet<TypeQualifier>),
    /// CONST X = 42
    Constant(TypeQualifier),
    /// DIM X AS INTEGER
    ExtendedBuiltIn(TypeQualifier),
    /// DIM X AS Card
    UserDefined(BareName),
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
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    sub_program: Option<(BareName, SubProgramType)>,
    names: HashMap<BareName, ResolvedTypeDefinitions>,
    user_defined_types: Rc<ResolvedUserDefinedTypes>,

    /// Collects names of variables and parameters whose type is a user defined type.
    /// These names cannot exist elsewhere as a prefix of a dotted variable, constant, parameter, function or sub name,
    /// regardless of the scope.
    ///
    /// This hash set exists only on the parent level.
    pub names_without_dot: Option<HashSet<BareName>>,
}

impl LinterContext {
    pub fn new(user_defined_types: Rc<ResolvedUserDefinedTypes>) -> Self {
        Self {
            parent: None,
            sub_program: None,
            names: HashMap::new(),
            user_defined_types,
            names_without_dot: Some(HashSet::new()),
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
            Some(ResolvedElement { element_type, .. }) => match element_type {
                ResolvedElementType::Integer
                | ResolvedElementType::Long
                | ResolvedElementType::Single
                | ResolvedElementType::Double
                | ResolvedElementType::String(_) => {
                    if rest.is_empty() {
                        Ok(Members::Leaf {
                            name: first.clone(),
                            element_type: element_type.clone(),
                        })
                    } else {
                        Err(QError::syntax_error("Cannot navigate after built-in type"))
                    }
                }
                ResolvedElementType::UserDefined(u) => {
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
            None => Err(QError::syntax_error("Element not defined")),
        }
    }

    fn resolve_member(&self, name: &BareName) -> Result<Option<ResolvedDeclaredName>, QError> {
        let s: String = name.clone().into();
        let mut v: Vec<BareName> = s.split('.').map(|s| s.into()).collect();
        let first: BareName = v.remove(0);
        match self.get_user_defined_type_name(&first) {
            Some(type_name) => {
                if v.is_empty() {
                    Ok(Some(ResolvedDeclaredName::UserDefined(UserDefinedName {
                        name: first.clone(),
                        type_name: type_name.clone(),
                    })))
                } else {
                    Ok(Some(ResolvedDeclaredName::Many(
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
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<(ResolvedDeclaredName, bool), QError> {
        let DeclaredName {
            name,
            type_definition,
        } = declared_name;
        match type_definition {
            TypeDefinition::Bare => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                let q: TypeQualifier = (&name).resolve_into(resolver);
                Ok((
                    ResolvedDeclaredName::BuiltIn(QualifiedName::new(name, q)),
                    false,
                ))
            }
            TypeDefinition::CompactBuiltIn(q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((
                    ResolvedDeclaredName::BuiltIn(QualifiedName::new(name, q)),
                    false,
                ))
            }
            TypeDefinition::ExtendedBuiltIn(q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((
                    ResolvedDeclaredName::BuiltIn(QualifiedName::new(name, q)),
                    true,
                ))
            }
            TypeDefinition::UserDefined(user_defined_type) => {
                if name.contains('.') {
                    Err(QError::syntax_error("Identifier cannot include period"))
                } else if self.user_defined_types.contains_key(&user_defined_type) {
                    self.ensure_user_defined_var_not_clashing_with_dotted_vars(&name)?;
                    Ok((
                        ResolvedDeclaredName::UserDefined(UserDefinedName {
                            name,
                            type_name: user_defined_type,
                        }),
                        true,
                    ))
                } else {
                    Err(QError::syntax_error("Type not defined"))
                }
            }
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.names.contains_key(bare_name.as_ref())
    }

    pub fn push_param<T: TypeResolver>(
        &mut self,
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<(), QError> {
        self.push_dim(declared_name, resolver).map(|_| ())
    }

    fn push_resolved_const(&mut self, q_name: QualifiedName) -> Result<(), QError> {
        if self.contains_any(&q_name) {
            Err(QError::DuplicateDefinition)
        } else {
            let QualifiedName { name, qualifier } = q_name;
            self.names
                .insert(name, ResolvedTypeDefinitions::Constant(qualifier));
            Ok(())
        }
    }

    pub fn push_dim<T: TypeResolver>(
        &mut self,
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<ResolvedDeclaredName, QError> {
        let (resolved_declared_name, is_extended) =
            self.resolve_declared_name(declared_name, resolver)?;
        let name: &BareName = resolved_declared_name.as_ref();
        match self.names.get_mut(&name) {
            Some(resolved_type_definitions) => {
                match resolved_type_definitions {
                    ResolvedTypeDefinitions::Compact(existing_set) => {
                        match resolved_declared_name.type_definition() {
                            ResolvedTypeDefinition::BuiltIn(q) => {
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
            None => match resolved_declared_name.type_definition() {
                ResolvedTypeDefinition::BuiltIn(q) => {
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
                // TODO support top level DIM x AS STRING * 6
                ResolvedTypeDefinition::String(_) => {
                    self.names.insert(
                        name.clone(),
                        ResolvedTypeDefinitions::ExtendedBuiltIn(TypeQualifier::DollarString),
                    );
                }
                ResolvedTypeDefinition::UserDefined(u) => {
                    self.collect_name_without_dot(name);
                    self.names.insert(
                        name.clone(),
                        ResolvedTypeDefinitions::UserDefined(u.clone()),
                    );
                }
            },
        }
        Ok(resolved_declared_name)
    }

    fn do_resolve_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<ResolvedDeclaredName>, QError> {
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
                                    Ok(Some(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                                        b.clone(),
                                        qualifier,
                                    ))))
                                } else {
                                    Ok(None)
                                }
                            }
                            Name::Qualified { name, qualifier } => {
                                if existing_set.contains(qualifier) {
                                    Ok(Some(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                                        name.clone(),
                                        *qualifier,
                                    ))))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::ExtendedBuiltIn(q) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => Ok(Some(ResolvedDeclaredName::BuiltIn(
                                QualifiedName::new(b.clone(), *q),
                            ))),
                            Name::Qualified { name, qualifier } => {
                                if q == qualifier {
                                    Ok(Some(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                                        name.clone(),
                                        *qualifier,
                                    ))))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::UserDefined(u) => {
                        // only possible if the name is bare
                        match name {
                            Name::Bare(b) => {
                                Ok(Some(ResolvedDeclaredName::UserDefined(UserDefinedName {
                                    name: b.clone(),
                                    type_name: u.clone(),
                                })))
                            }
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
                            Ok(Some(Expression::Variable(ResolvedDeclaredName::BuiltIn(
                                QualifiedName::new(b.clone(), qualifier),
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                    Name::Qualified { name, qualifier } => {
                        if existing_set.contains(qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(ResolvedDeclaredName::BuiltIn(
                                QualifiedName::new(name.clone(), *qualifier),
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
                        Ok(Some(Expression::Variable(ResolvedDeclaredName::BuiltIn(
                            QualifiedName::new(bare_name.clone(), *q),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                ResolvedTypeDefinitions::UserDefined(u) => {
                    if name.is_bare() {
                        // TODO fix me
                        Ok(Some(Expression::Variable(
                            ResolvedDeclaredName::UserDefined(UserDefinedName {
                                name: bare_name.clone(),
                                type_name: u.clone(),
                            }),
                        )))
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
    ) -> Result<ResolvedDeclaredName, QError> {
        let QualifiedName { name, qualifier } = name.resolve_into(resolver);
        match self.names.get_mut(name.as_ref()) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                ResolvedTypeDefinitions::Compact(existing_set) => {
                    if existing_set.contains(&qualifier) {
                        Err(QError::DuplicateDefinition)
                    } else {
                        existing_set.insert(qualifier);
                        Ok(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                            name, qualifier,
                        )))
                    }
                }
                _ => Err(QError::DuplicateDefinition),
            },
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(qualifier);
                self.names
                    .insert(name.clone(), ResolvedTypeDefinitions::Compact(s));
                Ok(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                    name, qualifier,
                )))
            }
        }
    }

    pub fn resolve_missing_name_in_expression<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Expression, QError> {
        let resolved_declared_names = self.resolve_missing_name_in_assignment(name, resolver)?;
        Ok(Expression::Variable(resolved_declared_names))
    }

    pub fn push_function_context(self, name: &BareName) -> Self {
        let mut result = Self {
            parent: None,
            sub_program: Some((name.clone(), SubProgramType::Function)),
            names: HashMap::new(),
            user_defined_types: Rc::clone(&self.user_defined_types),
            names_without_dot: None,
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn push_sub_context(self, name: &BareName) -> Self {
        let mut result = Self {
            parent: None,
            sub_program: Some((name.clone(), SubProgramType::Sub)),
            names: HashMap::new(),
            user_defined_types: Rc::clone(&self.user_defined_types),
            names_without_dot: None,
        };
        result.parent = Some(Box::new(self));
        result
    }

    pub fn pop_context(self) -> Self {
        *self.parent.expect("Stack underflow!")
    }

    pub fn push_const(
        &mut self,
        name: Name,
        right_side_type: Locatable<TypeQualifier>,
    ) -> Result<QualifiedName, QErrorNode> {
        let Locatable {
            element: right_side_q,
            pos: right_side_pos,
        } = right_side_type;
        let q = match &name {
            // bare name resolves from right side, not resolver
            Name::Bare(_) => right_side_q,
            Name::Qualified { qualifier, .. } => {
                if right_side_q.can_cast_to(*qualifier) {
                    *qualifier
                } else {
                    return Err(QError::TypeMismatch).with_err_at(right_side_pos);
                }
            }
        };
        let q_name = name.with_type(q);
        self.push_resolved_const(q_name.clone()).with_err_no_pos()?;
        Ok(q_name)
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
    ) -> Result<ResolvedDeclaredName, QError> {
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
