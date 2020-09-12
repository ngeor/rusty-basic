use crate::common::{Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::types::{
    Expression, ResolvedDeclaredName, ResolvedDeclaredNames, ResolvedElement, ResolvedElementType,
    ResolvedTypeDefinition, ResolvedUserDefinedType,
};
use crate::parser::{
    BareName, CanCastTo, DeclaredName, Name, QualifiedName, TypeDefinition, TypeQualifier,
    WithTypeQualifier,
};
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

#[derive(Debug, Default)]
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    sub_program: Option<(BareName, SubProgramType)>,
    names: HashMap<BareName, ResolvedTypeDefinitions>,
    pub user_defined_types: HashMap<BareName, ResolvedUserDefinedType>,
}

impl LinterContext {
    fn get_user_defined_type_name(&self, name: &BareName) -> Option<&BareName> {
        match self.names.get(name) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                ResolvedTypeDefinitions::UserDefined(type_name) => Some(type_name),
                _ => None,
            },
            None => None,
        }
    }

    fn resolve_member(&self, name: &BareName) -> Result<Vec<ResolvedDeclaredName>, QError> {
        let s: String = name.clone().into();
        let v: Vec<&str> = s.split('.').collect();
        let first: BareName = v[0].into();
        match self.get_user_defined_type_name(&first) {
            Some(type_name) => {
                let mut result: Vec<ResolvedDeclaredName> = vec![];
                result.push(ResolvedDeclaredName::new(
                    first,
                    ResolvedTypeDefinition::UserDefined(type_name.clone()),
                ));
                if v.len() > 1 {
                    // the first part matched, the rest must match too
                    let second: BareName = v[1].into();
                    let user_defined_type: &ResolvedUserDefinedType = self
                        .user_defined_types
                        .get(type_name)
                        .expect("should have found user defined type");
                    match user_defined_type.find_element(&second) {
                        Some(ResolvedElement { element_type, .. }) => {
                            let second_type_definition = match element_type {
                                ResolvedElementType::Integer => {
                                    ResolvedTypeDefinition::BuiltIn(TypeQualifier::PercentInteger)
                                }
                                ResolvedElementType::Long => {
                                    ResolvedTypeDefinition::BuiltIn(TypeQualifier::AmpersandLong)
                                }
                                ResolvedElementType::Single => {
                                    ResolvedTypeDefinition::BuiltIn(TypeQualifier::BangSingle)
                                }
                                ResolvedElementType::Double => {
                                    ResolvedTypeDefinition::BuiltIn(TypeQualifier::HashDouble)
                                }
                                ResolvedElementType::String(_) => {
                                    ResolvedTypeDefinition::BuiltIn(TypeQualifier::DollarString)
                                }
                                ResolvedElementType::UserDefined(_) => unimplemented!(),
                            };
                            result.push(ResolvedDeclaredName::new(second, second_type_definition));
                        }
                        None => {
                            // trying to reference A.Something where Something is not a member of the type of A
                            return Err(QError::syntax_error("Element not defined"));
                        }
                    }
                }
                Ok(result)
            }
            None => {
                // No user defined variable starts with the first dotted name
                Ok(vec![])
            }
        }
    }

    fn ensure_not_clashing_with_user_defined_var(&self, name: &BareName) -> Result<(), QError> {
        if name.contains('.') {
            let s: String = name.clone().into();
            let v: Vec<&str> = s.split('.').collect();
            let first: BareName = v[0].into();
            match self.get_user_defined_type_name(&first) {
                Some(_) => Err(QError::syntax_error("Expected: , or end-of-statement")),
                None => Ok(()),
            }
        } else {
            Ok(())
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
                    ResolvedDeclaredName {
                        name,
                        type_definition: ResolvedTypeDefinition::BuiltIn(q),
                    },
                    false,
                ))
            }
            TypeDefinition::CompactBuiltIn(q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((
                    ResolvedDeclaredName {
                        name,
                        type_definition: ResolvedTypeDefinition::BuiltIn(q),
                    },
                    false,
                ))
            }
            TypeDefinition::ExtendedBuiltIn(q) => {
                self.ensure_not_clashing_with_user_defined_var(&name)?;
                Ok((
                    ResolvedDeclaredName {
                        name,
                        type_definition: ResolvedTypeDefinition::BuiltIn(q),
                    },
                    true,
                ))
            }
            TypeDefinition::UserDefined(user_defined_type) => {
                if name.contains('.') {
                    Err(QError::syntax_error("Identifier cannot include period"))
                } else if self.user_defined_types.contains_key(&user_defined_type) {
                    self.ensure_user_defined_var_not_clashing_with_dotted_vars(&name)?;
                    Ok((
                        ResolvedDeclaredName {
                            name,
                            type_definition: ResolvedTypeDefinition::UserDefined(user_defined_type),
                        },
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
        let (
            ResolvedDeclaredName {
                name,
                type_definition,
            },
            is_extended,
        ) = self.resolve_declared_name(declared_name, resolver)?;
        match self.names.get_mut(&name) {
            Some(resolved_type_definitions) => {
                match resolved_type_definitions {
                    ResolvedTypeDefinitions::Compact(existing_set) => match &type_definition {
                        ResolvedTypeDefinition::BuiltIn(q) => {
                            if existing_set.contains(q) || is_extended {
                                return Err(QError::DuplicateDefinition);
                            } else {
                                existing_set.insert(*q);
                            }
                        }
                        _ => {
                            return Err(QError::DuplicateDefinition);
                        }
                    },
                    _ => {
                        // anything else cannot be extended
                        return Err(QError::DuplicateDefinition);
                    }
                }
            }
            None => match &type_definition {
                ResolvedTypeDefinition::BuiltIn(q) => {
                    if is_extended {
                        self.names
                            .insert(name.clone(), ResolvedTypeDefinitions::ExtendedBuiltIn(*q));
                    } else {
                        let mut s: HashSet<TypeQualifier> = HashSet::new();
                        s.insert(*q);
                        self.names
                            .insert(name.clone(), ResolvedTypeDefinitions::Compact(s));
                    }
                }
                ResolvedTypeDefinition::UserDefined(u) => {
                    self.names.insert(
                        name.clone(),
                        ResolvedTypeDefinitions::UserDefined(u.clone()),
                    );
                }
            },
        }
        Ok(ResolvedDeclaredName {
            name,
            type_definition,
        })
    }

    fn do_resolve_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<ResolvedDeclaredNames>, QError> {
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
                                    Ok(Some(ResolvedDeclaredName::single(
                                        b.clone(),
                                        ResolvedTypeDefinition::BuiltIn(qualifier),
                                    )))
                                } else {
                                    Ok(None)
                                }
                            }
                            Name::Qualified { name, qualifier } => {
                                if existing_set.contains(qualifier) {
                                    Ok(Some(ResolvedDeclaredName::single(
                                        name.clone(),
                                        ResolvedTypeDefinition::BuiltIn(*qualifier),
                                    )))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::ExtendedBuiltIn(q) => {
                        // only possible if the name is bare or using the same qualifier
                        match name {
                            Name::Bare(b) => Ok(Some(ResolvedDeclaredName::single(
                                b.clone(),
                                ResolvedTypeDefinition::BuiltIn(*q),
                            ))),
                            Name::Qualified { name, qualifier } => {
                                if q == qualifier {
                                    Ok(Some(ResolvedDeclaredName::single(
                                        name.clone(),
                                        ResolvedTypeDefinition::BuiltIn(*q),
                                    )))
                                } else {
                                    Err(QError::DuplicateDefinition)
                                }
                            }
                        }
                    }
                    ResolvedTypeDefinitions::UserDefined(u) => {
                        // only possible if the name is bare
                        match name {
                            Name::Bare(b) => Ok(Some(ResolvedDeclaredName::single(
                                b.clone(),
                                ResolvedTypeDefinition::UserDefined(u.clone()),
                            ))),
                            _ => Err(QError::TypeMismatch),
                        }
                    }
                }
            }
            None => {
                let names = if name.is_bare() {
                    self.resolve_member(bare_name)?
                } else {
                    vec![]
                };
                if names.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(names))
                }
            }
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
                            Ok(Some(Expression::Variable(ResolvedDeclaredName::single(
                                b.clone(),
                                ResolvedTypeDefinition::BuiltIn(qualifier),
                            ))))
                        } else {
                            Ok(None)
                        }
                    }
                    Name::Qualified { name, qualifier } => {
                        if existing_set.contains(qualifier) {
                            // TODO fix me
                            Ok(Some(Expression::Variable(ResolvedDeclaredName::single(
                                name.clone(),
                                ResolvedTypeDefinition::BuiltIn(*qualifier),
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
                        Ok(Some(Expression::Variable(ResolvedDeclaredName::single(
                            bare_name.clone(),
                            ResolvedTypeDefinition::BuiltIn(*q),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
                ResolvedTypeDefinitions::UserDefined(u) => {
                    if name.is_bare() {
                        // TODO fix me
                        Ok(Some(Expression::Variable(ResolvedDeclaredName::single(
                            bare_name.clone(),
                            ResolvedTypeDefinition::UserDefined(u.clone()),
                        ))))
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
            },
            None => {
                let names = if name.is_bare() {
                    self.resolve_member(bare_name)?
                } else {
                    vec![]
                };
                if names.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Expression::Variable(names)))
                }
            }
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
    ) -> Result<ResolvedDeclaredNames, QError> {
        let QualifiedName { name, qualifier } = name.resolve_into(resolver);
        match self.names.get_mut(name.as_ref()) {
            Some(resolved_type_definitions) => match resolved_type_definitions {
                ResolvedTypeDefinitions::Compact(existing_set) => {
                    if existing_set.contains(&qualifier) {
                        Err(QError::DuplicateDefinition)
                    } else {
                        existing_set.insert(qualifier);
                        Ok(ResolvedDeclaredName::single(
                            name,
                            ResolvedTypeDefinition::BuiltIn(qualifier),
                        ))
                    }
                }
                _ => Err(QError::DuplicateDefinition),
            },
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(qualifier);
                self.names
                    .insert(name.clone(), ResolvedTypeDefinitions::Compact(s));
                Ok(ResolvedDeclaredName::single(
                    name,
                    ResolvedTypeDefinition::BuiltIn(qualifier),
                ))
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
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.sub_program = Some((name.clone(), SubProgramType::Function));
        result
    }

    pub fn push_sub_context(self, name: &BareName) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.sub_program = Some((name.clone(), SubProgramType::Sub));
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
    ) -> Result<ResolvedDeclaredNames, QError> {
        match self.do_resolve_assignment(name, resolver)? {
            Some(x) => Ok(x),
            None => {
                // maybe a parent constant?
                match self.resolve_parent_const_expression(name)? {
                    Some(_) => Err(QError::DuplicateDefinition),
                    None => {
                        // just insert it
                        self.resolve_missing_name_in_assignment(name, resolver)
                    }
                }
            }
        }
    }
}
