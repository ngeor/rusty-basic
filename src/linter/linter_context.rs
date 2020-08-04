use crate::common::*;
use crate::linter::error::*;
use crate::linter::Expression;
use crate::parser::{
    BareName, HasQualifier, Name, NameNode, NameTrait, QualifiedName, TypeQualifier, TypeResolver,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    names: HashMap<CaseInsensitiveString, Vec<Identifier>>,
    // TODO replace with one sub_program_name
    function_name: Option<CaseInsensitiveString>,
    sub_name: Option<CaseInsensitiveString>,
}

#[derive(Debug)]
pub enum Identifier {
    Constant(TypeQualifier),
    /// A variable which was declared with DIM X AS type
    ExplicitVar(TypeQualifier),
    /// A variable which was declared implicitly e.g. A$ = "hello" or with DIM A$
    ImplicitVar(TypeQualifier),
    /// A sub or function parameter. This can hide constants.
    Param(TypeQualifier),
}

impl Identifier {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            _ => false,
        }
    }

    pub fn is_explicit_var(&self) -> bool {
        match self {
            Self::ExplicitVar(_) => true,
            _ => false,
        }
    }

    pub fn is_implicit_var(&self) -> bool {
        match self {
            Self::ImplicitVar(_) => true,
            _ => false,
        }
    }

    pub fn is_param(&self) -> bool {
        match self {
            Self::Param(_) => true,
            _ => false,
        }
    }
}

impl HasQualifier for Identifier {
    fn qualifier(&self) -> TypeQualifier {
        match self {
            Self::Constant(q) | Self::ExplicitVar(q) | Self::ImplicitVar(q) | Self::Param(q) => *q,
        }
    }
}

impl LinterContext {
    pub fn push_function_context(self, name: &CaseInsensitiveString) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.function_name = Some(name.clone());
        result
    }

    pub fn push_sub_context(self, name: &CaseInsensitiveString) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.sub_name = Some(name.clone());
        result
    }

    pub fn pop_context(self) -> Self {
        *self.parent.expect("Stack underflow!")
    }

    pub fn add_param(&mut self, name: QualifiedName) -> Result<(), Error> {
        let bare_name = name.bare_name().clone();
        let q = name.qualifier();
        match self.names.get_mut(&bare_name) {
            Some(v) => v.push(Identifier::Param(q)),
            None => {
                self.names.insert(bare_name, vec![Identifier::Param(q)]);
                ()
            }
        };
        Ok(())
    }

    pub fn add_const(
        &mut self,
        name_node: NameNode,
        right_side_type: Locatable<TypeQualifier>,
    ) -> Result<TypeQualifier, Error> {
        let bare_name = name_node.bare_name().clone();
        match self.names.get_mut(&bare_name) {
            Some(_) => err_l(LinterError::DuplicateDefinition, &name_node),
            None => {
                let q = match name_node.as_ref() {
                    // bare name resolves from right side, not resolver
                    Name::Bare(b) => *right_side_type.as_ref(),
                    Name::Qualified(q) => {
                        if right_side_type.as_ref().can_cast_to(q.qualifier()) {
                            q.qualifier()
                        } else {
                            return err_l(LinterError::TypeMismatch, &right_side_type);
                        }
                    }
                };
                self.names.insert(bare_name, vec![Identifier::Constant(q)]);
                Ok(q)
            }
        }
    }

    // e.g. DIM A, DIM A$
    pub fn add_dim_implicit<T: TypeResolver>(
        &mut self,
        name: Name,
        resolver: &T,
    ) -> Result<TypeQualifier, Error> {
        let q = match &name {
            Name::Bare(b) => resolver.resolve(b),
            Name::Qualified(q) => q.qualifier(),
        };
        match self.names.get_mut(name.bare_name()) {
            Some(v) => {
                for i in v.iter() {
                    match i {
                        Identifier::Constant(_)
                        | Identifier::ExplicitVar(_)
                        | Identifier::Param(_) => {
                            return err_no_pos(LinterError::DuplicateDefinition)
                        }
                        Identifier::ImplicitVar(q_existing) => {
                            if q == *q_existing {
                                return err_no_pos(LinterError::DuplicateDefinition);
                            }
                        }
                    }
                }
                v.push(Identifier::ImplicitVar(q));
            }
            None => {
                self.names
                    .insert(name.bare_name().clone(), vec![Identifier::ImplicitVar(q)]);
            }
        };
        Ok(q)
    }

    pub fn add_dim_implicit_implicit<T: TypeResolver>(
        &mut self,
        name: Name,
        resolver: &T,
    ) -> Result<TypeQualifier, Error> {
        let q = match &name {
            Name::Bare(b) => resolver.resolve(b),
            Name::Qualified(q) => q.qualifier(),
        };
        match self.names.get_mut(name.bare_name()) {
            Some(v) => {
                let mut already_exists = false;
                for i in v.iter() {
                    match i {
                        Identifier::Constant(_)
                        | Identifier::ExplicitVar(_)
                        | Identifier::Param(_) => {
                            return err_no_pos(LinterError::DuplicateDefinition)
                        }
                        Identifier::ImplicitVar(q_existing) => {
                            if q == *q_existing {
                                already_exists = true;
                                break;
                            }
                        }
                    }
                }
                if !already_exists {
                    v.push(Identifier::ImplicitVar(q));
                }
            }
            None => {
                self.names
                    .insert(name.bare_name().clone(), vec![Identifier::ImplicitVar(q)]);
            }
        };
        Ok(q)
    }

    pub fn add_dim_explicit(&mut self, bare_name: BareName, q: TypeQualifier) -> Result<(), Error> {
        match self.names.get_mut(&bare_name) {
            Some(_) => err_no_pos(LinterError::DuplicateDefinition),
            None => {
                self.names
                    .insert(bare_name, vec![Identifier::ExplicitVar(q)]);
                Ok(())
            }
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        n: Name,
        resolver: &T,
    ) -> Result<QualifiedName, Error> {
        let blank: Vec<Identifier> = vec![];
        let identifiers = self.names.get(n.bare_name()).unwrap_or(&blank);
        if identifiers.iter().any(|x| x.is_constant()) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else if identifiers.iter().any(|x| x.is_explicit_var()) {
            // if we use a bare name or we use the correct type qualifier, it is allowed
            let q = identifiers
                .iter()
                .find(|x| x.is_explicit_var())
                .unwrap()
                .qualifier();
            match n {
                Name::Bare(b) => Ok(QualifiedName::new(b, q)),
                Name::Qualified(q_name) => {
                    if q_name.qualifier() == q {
                        Ok(q_name)
                    } else {
                        err_no_pos(LinterError::DuplicateDefinition)
                    }
                }
            }
        } else if identifiers.iter().any(|x| x.is_param()) {
            let q = identifiers
                .iter()
                .find(|x| x.is_param())
                .unwrap()
                .qualifier();
            match n {
                Name::Bare(b) => Ok(QualifiedName::new(b, q)),
                Name::Qualified(q_name) => {
                    if q_name.qualifier() == q {
                        Ok(q_name)
                    } else {
                        err_no_pos(LinterError::DuplicateDefinition)
                    }
                }
            }
        } else {
            let q = match &n {
                Name::Bare(b) => resolver.resolve(b),
                Name::Qualified(q) => q.qualifier(),
            };
            let result = QualifiedName::new(n.bare_name().clone(), q);
            self.add_dim_implicit_implicit(n, resolver)?;
            Ok(result)
        }
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        n: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, Error> {
        match self.names.get(n.bare_name()) {
            Some(v) => {
                // try parameters
                match v.iter().find(|x| x.is_param()).map(|x| x.qualifier()) {
                    Some(q) => {
                        if n.bare_or_eq(q) {
                            return Ok(Some(Expression::Variable(QualifiedName::new(
                                n.bare_name().clone(),
                                q,
                            ))));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    }
                    None => {}
                }
                // try constants
                match v.iter().find(|x| x.is_constant()).map(|x| x.qualifier()) {
                    Some(q) => {
                        if n.bare_or_eq(q) {
                            return Ok(Some(Expression::Constant(QualifiedName::new(
                                n.bare_name().clone(),
                                q,
                            ))));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    }
                    None => {}
                }
                // try explicit variables
                match v
                    .iter()
                    .find(|x| x.is_explicit_var())
                    .map(|x| x.qualifier())
                {
                    Some(q) => {
                        if n.bare_or_eq(q) {
                            return Ok(Some(Expression::Variable(QualifiedName::new(
                                n.bare_name().clone(),
                                q,
                            ))));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    }
                    None => {}
                }
                // no need to check for implicit variables because Converter always adds them back
            }
            None => {}
        }

        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        match self.names.get(n.bare_name()) {
            Some(v) => {
                for i in v {
                    match i {
                        Identifier::Constant(q) => {
                            if n.bare_or_eq(*q) {
                                return Ok(Some(Expression::Constant(QualifiedName::new(
                                    n.bare_name().clone(),
                                    *q,
                                ))));
                            } else {
                                return err_no_pos(LinterError::DuplicateDefinition);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        }

        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    // pub fn get_constant_type(&self, n: &Name) -> Result<Option<TypeQualifier>, Error> {
    //     let bare_name: &CaseInsensitiveString = n.bare_name();
    //     match self.constants.get(bare_name) {
    //         Some(const_type) => {
    //             // it's okay to reference a const unqualified
    //             if n.bare_or_eq(*const_type) {
    //                 Ok(Some(*const_type))
    //             } else {
    //                 Err(LinterError::DuplicateDefinition.into())
    //             }
    //         }
    //         None => Ok(None),
    //     }
    // }

    // pub fn get_parent_constant_type(&self, n: &Name) -> Result<Option<TypeQualifier>, Error> {
    //     match &self.parent {
    //         Some(p) => {
    //             let x = p.get_constant_type(n)?;
    //             match x {
    //                 Some(q) => Ok(Some(q)),
    //                 None => p.get_parent_constant_type(n),
    //             }
    //         }
    //         None => Ok(None),
    //     }
    // }

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.function_name {
            Some(x) => x == name.bare_name(),
            None => false,
        }
    }
}
