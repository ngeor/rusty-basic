use crate::common::*;
use crate::linter::error::*;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::Expression;
use crate::parser::{
    BareName, HasQualifier, Name, NameNode, NameTrait, QualifiedName, TypeQualifier,
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
enum Identifier {
    Constant(TypeQualifier),
    /// A variable which was declared with DIM X AS type
    ExtendedVar(TypeQualifier),
    /// A variable which was declared implicitly e.g. A$ = "hello" or with DIM A$
    CompactVar(TypeQualifier),
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

    pub fn is_extended_var(&self) -> bool {
        match self {
            Self::ExtendedVar(_) => true,
            _ => false,
        }
    }

    pub fn is_compact_var(&self) -> bool {
        match self {
            Self::CompactVar(_) => true,
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
            Self::Constant(q) | Self::ExtendedVar(q) | Self::CompactVar(q) | Self::Param(q) => *q,
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

    fn get_names(&mut self, n: &BareName) -> &mut Vec<Identifier> {
        if !self.names.contains_key(n) {
            self.names.insert(n.clone(), vec![]);
        }
        self.names.get_mut(n).unwrap()
    }

    pub fn add_param(&mut self, name: QualifiedName) -> Result<(), Error> {
        let (bare_name, q) = name.consume();
        let identifiers = self.get_names(&bare_name);
        // should not have multiple parameters of the same name
        // TODO check if it is allowed to have params with same bare name but different qualifier
        if identifiers.iter().any(|x| x.is_param()) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            identifiers.push(Identifier::Param(q));
            Ok(())
        }
    }

    pub fn add_const(
        &mut self,
        name_node: NameNode,
        right_side_type: Locatable<TypeQualifier>,
    ) -> Result<TypeQualifier, Error> {
        let identifiers = self.get_names(name_node.bare_name());
        if identifiers.is_empty() {
            let q = match name_node.as_ref() {
                // bare name resolves from right side, not resolver
                Name::Bare(_) => *right_side_type.as_ref(),
                Name::Qualified(q) => {
                    if right_side_type.as_ref().can_cast_to(q.qualifier()) {
                        q.qualifier()
                    } else {
                        return err_l(LinterError::TypeMismatch, &right_side_type);
                    }
                }
            };
            identifiers.push(Identifier::Constant(q));
            Ok(q)
        } else {
            err_l(LinterError::DuplicateDefinition, &name_node)
        }
    }

    // e.g. DIM A, DIM A$
    pub fn add_dim_compact<T: TypeResolver>(
        &mut self,
        name: Name,
        resolver: &T,
    ) -> Result<TypeQualifier, Error> {
        let q = resolver.resolve(&name);
        let identifiers = self.get_names(name.bare_name());
        if Self::can_add_compact(&identifiers, q)? {
            identifiers.push(Identifier::CompactVar(q));
            Ok(q)
        } else {
            err_no_pos(LinterError::DuplicateDefinition)
        }
    }

    pub fn add_dim_compact_implicit<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<TypeQualifier, Error> {
        let q = resolver.resolve(name);
        let identifiers = self.get_names(name.bare_name());
        if Self::can_add_compact(&identifiers, q)? {
            identifiers.push(Identifier::CompactVar(q));
        }
        Ok(q)
    }

    fn can_add_compact(identifiers: &Vec<Identifier>, new_q: TypeQualifier) -> Result<bool, Error> {
        let mut already_exists = false;
        for i in identifiers.iter() {
            match i {
                Identifier::Constant(_)
                | Identifier::ExtendedVar(_)
                | Identifier::Param(_) => {
                    return err_no_pos(LinterError::DuplicateDefinition)
                }
                Identifier::CompactVar(q_existing) => {
                    if new_q == *q_existing {
                        already_exists = true;
                    }
                }
            }
        }
        Ok(!already_exists)
    }

    pub fn add_dim_extended(&mut self, bare_name: BareName, q: TypeQualifier) -> Result<(), Error> {
        let identifiers = self.get_names(&bare_name);
        if identifiers.is_empty() {
            identifiers.push(Identifier::ExtendedVar(q));
            Ok(())
        } else {
            err_no_pos(LinterError::DuplicateDefinition)
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        n: &Name,
        resolver: &T,
    ) -> Result<QualifiedName, Error> {
        let blank: Vec<Identifier> = vec![];
        let identifiers = self.names.get(n.bare_name()).unwrap_or(&blank);
        Self::resolve_assignment_const(identifiers)
        .or_try_read(|| Self::resolve_assignment_extended_var(identifiers, n) )
        .or_try_read(|| Self::resolve_assignment_param(identifiers, n))
        .or_read(|| self.resolve_assignment_implicit(n, resolver))
    }

    fn resolve_assignment_const(identifiers: &Vec<Identifier>) -> Result<Option<QualifiedName>, Error> {
        if identifiers.iter().any(|x| x.is_constant()) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            Ok(None)
        }
    }

    fn resolve_assignment_extended_var(identifiers: &Vec<Identifier>, n: &Name) -> Result<Option<QualifiedName>, Error> {
        match identifiers.iter().find(|x| x.is_extended_var()) {
            Some(i) => {
                let q = i.qualifier();
                match n {
                    Name::Bare(_) => Ok(Some(n.with_type_ref(q))),
                    Name::Qualified(q_name) => {
                        if q_name.qualifier() == q {
                            Ok(Some(q_name.clone()))
                        } else {
                            err_no_pos(LinterError::DuplicateDefinition)
                        }
                    }
                }
            }
            None => Ok(None)
        }
    }

    fn resolve_assignment_param(identifiers: &Vec<Identifier>, n: &Name) -> Result<Option<QualifiedName>, Error> {
        match identifiers.iter().find(|x| x.is_param()) {
            Some(i) => {
                let q = i.qualifier();
                match n {
                    Name::Bare(_) => Ok(Some(n.with_type_ref(q))),
                    Name::Qualified(q_name) => {
                        if q_name.qualifier() == q {
                            Ok(Some(q_name.clone()))
                        } else {
                            err_no_pos(LinterError::DuplicateDefinition)
                        }
                    }
                }
            }
            None => Ok(None)
        }
    }

    fn resolve_assignment_implicit<T: TypeResolver>(&mut self, n: &Name, resolver: &T) -> Result<QualifiedName, Error> {
        let result = resolver.to_qualified_name(n);
        self.add_dim_compact_implicit(n, resolver)?;
        Ok(result)
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
                            return Ok(Some(Expression::Variable(n.with_type_ref(q))));
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
                            return Ok(Some(Expression::Constant(n.with_type_ref(q))));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    }
                    None => {}
                }
                // try extended variables
                match v
                    .iter()
                    .find(|x| x.is_extended_var())
                    .map(|x| x.qualifier())
                {
                    Some(q) => {
                        if n.bare_or_eq(q) {
                            return Ok(Some(Expression::Variable(n.with_type_ref(q))));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    }
                    None => {}
                }
                // try compact variable
                match v.iter().find(|x| x.is_compact_var()).map(|x| x.qualifier()) {
                    Some(q_declared) => {
                        let q = resolver.resolve(n);
                        if q == q_declared {
                            return Ok(Some(Expression::Variable(n.with_type_ref(q))));
                        }
                    }
                    None => {}
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

    fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        match self.names.get(n.bare_name()) {
            Some(v) => {
                for i in v {
                    match i {
                        Identifier::Constant(q) => {
                            if n.bare_or_eq(*q) {
                                return Ok(Some(Expression::Constant(n.with_type_ref(*q))));
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

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.function_name {
            Some(x) => x == name.bare_name(),
            None => false,
        }
    }
}
