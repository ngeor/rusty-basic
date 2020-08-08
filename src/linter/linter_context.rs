use crate::common::*;
use crate::linter::error::*;
use crate::linter::type_resolver::*;
use crate::linter::Expression;
use crate::parser::{
    BareName, DeclaredName, HasQualifier, Name, NameNode, QualifiedName, TypeDefinition,
    TypeQualifier, WithTypeQualifier,
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

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
    Variable(TypeDefinition),
    Param(TypeDefinition),
}

impl Identifier {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            _ => false,
        }
    }

    pub fn is_param(&self) -> bool {
        match self {
            Self::Param(_) => true,
            _ => false,
        }
    }

    pub fn is_extended_param(&self) -> bool {
        match self {
            Self::Param(t) => t.is_extended(),
            _ => false,
        }
    }

    pub fn is_compact_param_of_type(&self, q: TypeQualifier) -> bool {
        match self {
            Self::Param(t) => t.is_compact_of_type(q),
            _ => false,
        }
    }

    pub fn is_variable(&self) -> bool {
        match self {
            Self::Variable(_) => true,
            _ => false,
        }
    }

    pub fn is_extended_variable(&self) -> bool {
        match self {
            Self::Variable(t) => t.is_extended(),
            _ => false,
        }
    }

    pub fn is_compact_variable_of_type(&self, q: TypeQualifier) -> bool {
        match self {
            Self::Variable(t) => t.is_compact_of_type(q),
            _ => false,
        }
    }
}

impl TryFrom<&Identifier> for TypeQualifier {
    type Error = bool;
    fn try_from(i: &Identifier) -> Result<TypeQualifier, bool> {
        match i {
            Identifier::Constant(q) => Ok(*q),
            Identifier::Param(type_def) | Identifier::Variable(type_def) => type_def.try_into(),
        }
    }
}

impl HasQualifier for Identifier {
    fn qualifier(&self) -> TypeQualifier {
        self.try_into()
            .expect("Not supported for user defined types")
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

    pub fn add_param<T: TypeResolver>(
        &mut self,
        name: DeclaredName,
        resolver: &T,
    ) -> Result<(), Error> {
        let bare_name: &BareName = name.as_ref();
        let identifiers = self.get_names(bare_name);
        let resolved_q: TypeQualifier;
        match name.type_definition() {
            TypeDefinition::Bare => {
                // need resolver
                resolved_q = bare_name.resolve_into(resolver);
                // it's not allowed to have an extended param of this name or another compact param of the same type
                if identifiers
                    .iter()
                    .any(|x| x.is_extended_param() || x.is_compact_param_of_type(resolved_q))
                {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Param(TypeDefinition::CompactBuiltIn(
                    resolved_q,
                )));
            }
            TypeDefinition::CompactBuiltIn(q) => {
                resolved_q = *q;
                // it's not allowed to have an extended param of this name or another compact param of the same type
                if identifiers
                    .iter()
                    .any(|x| x.is_extended_param() || x.is_compact_param_of_type(resolved_q))
                {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Param(TypeDefinition::CompactBuiltIn(
                    resolved_q,
                )));
            }
            TypeDefinition::ExtendedBuiltIn(q) => {
                // it's not allowed to have any other params of this name
                if identifiers.iter().any(|x| x.is_param()) {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Param(TypeDefinition::ExtendedBuiltIn(*q)));
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    pub fn add_const(
        &mut self,
        name_node: NameNode,
        right_side_type: Locatable<TypeQualifier>,
    ) -> Result<TypeQualifier, Error> {
        let identifiers = self.get_names(name_node.as_ref());
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

    // e.g. DIM A, DIM A$, DIM AS STRING
    pub fn add_dim<T: TypeResolver>(
        &mut self,
        name: &DeclaredName,
        resolver: &T,
    ) -> Result<DeclaredName, Error> {
        let bare_name: &BareName = name.as_ref();
        let identifiers = self.get_names(bare_name);
        let resolved_q: TypeQualifier;

        match name.type_definition() {
            TypeDefinition::Bare => {
                resolved_q = bare_name.resolve_into(resolver);
                if identifiers.iter().any(|x| {
                    x.is_extended_variable()
                        || x.is_compact_variable_of_type(resolved_q)
                        || !x.is_variable()
                }) {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Variable(TypeDefinition::CompactBuiltIn(
                    resolved_q,
                )));
                Ok(DeclaredName::compact(bare_name, resolved_q))
            }
            TypeDefinition::CompactBuiltIn(q) => {
                resolved_q = *q;
                if identifiers.iter().any(|x| {
                    x.is_extended_variable()
                        || x.is_compact_variable_of_type(resolved_q)
                        || !x.is_variable()
                }) {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Variable(TypeDefinition::CompactBuiltIn(
                    resolved_q,
                )));
                Ok(DeclaredName::compact(bare_name, resolved_q))
            }
            TypeDefinition::ExtendedBuiltIn(q) => {
                if !identifiers.is_empty() {
                    return err_no_pos(LinterError::DuplicateDefinition);
                }
                identifiers.push(Identifier::Variable(TypeDefinition::ExtendedBuiltIn(*q)));
                Ok(DeclaredName::new(
                    bare_name.clone(),
                    TypeDefinition::ExtendedBuiltIn(*q),
                ))
            }
            _ => unimplemented!(),
        }
    }

    pub fn add_dim_compact_implicit<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<TypeQualifier, Error> {
        let q: TypeQualifier = name.resolve_into(resolver);
        let identifiers = self.get_names(name.as_ref());
        let mut already_exists = false;
        for i in identifiers.iter() {
            match i {
                Identifier::Constant(_) => return err_no_pos(LinterError::DuplicateDefinition),
                Identifier::Variable(type_def) | Identifier::Param(type_def) => {
                    if type_def.is_extended() {
                        return err_no_pos(LinterError::DuplicateDefinition);
                    }
                    if type_def.is_compact_of_type(q) {
                        already_exists = true;
                    }
                }
            }
        }

        if !already_exists {
            identifiers.push(Identifier::Variable(TypeDefinition::CompactBuiltIn(q)));
        }
        Ok(q)
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        n: &Name,
        resolver: &T,
    ) -> Result<QualifiedName, Error> {
        let blank: Vec<Identifier> = vec![];
        let identifiers = self.names.get(n.as_ref()).unwrap_or(&blank);
        Self::resolve_assignment_const(identifiers)
            .or_try_read(|| Self::resolve_assignment_var_or_param(identifiers, n, resolver))
            .or_read(|| self.resolve_assignment_implicit(n, resolver))
    }

    fn resolve_assignment_const(
        identifiers: &Vec<Identifier>,
    ) -> Result<Option<QualifiedName>, Error> {
        if identifiers.iter().any(|x| x.is_constant()) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            Ok(None)
        }
    }

    fn resolve_assignment_var_or_param<T: TypeResolver>(
        identifiers: &Vec<Identifier>,
        n: &Name,
        resolver: &T,
    ) -> Result<Option<QualifiedName>, Error> {
        // try extended params & variables
        match identifiers
            .iter()
            .find(|x| x.is_extended_param() || x.is_extended_variable())
            .map(|x| x.qualifier())
        {
            Some(q) => match n {
                Name::Bare(_) => return Ok(Some(n.with_type_ref(q))),
                Name::Qualified(_) => return err_no_pos(LinterError::DuplicateDefinition),
            },
            None => {}
        }

        let q: TypeQualifier = n.resolve_into(resolver);
        match identifiers
            .iter()
            .find(|x| x.is_compact_param_of_type(q) || x.is_compact_variable_of_type(q))
            .map(|x| x.qualifier())
        {
            Some(q) => Ok(Some(n.with_type_ref(q))),
            None => Ok(None),
        }
    }

    fn resolve_assignment_implicit<T: TypeResolver>(
        &mut self,
        n: &Name,
        resolver: &T,
    ) -> Result<QualifiedName, Error> {
        let result: QualifiedName = n.resolve_into(resolver);
        self.add_dim_compact_implicit(n, resolver)?;
        Ok(result)
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        n: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, Error> {
        match n {
            Name::Bare(b) => self
                .resolve_expression_bare(b, resolver)
                .or_try_read(|| self.resolve_parent_const_expression(n)),
            Name::Qualified(q) => self
                .resolve_expression_qualified(q)
                .or_try_read(|| self.resolve_parent_const_expression(n)),
        }
    }

    fn resolve_expression_bare<T: TypeResolver>(
        &self,
        n: &BareName,
        resolver: &T,
    ) -> Result<Option<Expression>, Error> {
        let q_resolved = resolver.resolve(n);
        match self.names.get(n) {
            Some(v) => {
                // try extended parameters
                match v
                    .iter()
                    .find(|x| x.is_extended_param())
                    .map(|x| x.qualifier())
                {
                    Some(q) => return Ok(Some(Expression::Variable(n.with_type_ref(q)))),
                    None => {}
                }
                // try compact parameters
                match v
                    .iter()
                    .find(|x| x.is_compact_param_of_type(q_resolved))
                    .map(|x| x.qualifier())
                {
                    Some(q) => {
                        return Ok(Some(Expression::Variable(n.with_type_ref(q))));
                    }
                    None => {}
                }
                // try constants
                match v.iter().find(|x| x.is_constant()).map(|x| x.qualifier()) {
                    Some(q) => {
                        return Ok(Some(Expression::Constant(n.with_type_ref(q))));
                    }
                    None => {}
                }
                // try extended variables
                match v
                    .iter()
                    .find(|x| x.is_extended_variable())
                    .map(|x| x.qualifier())
                {
                    Some(q) => {
                        return Ok(Some(Expression::Variable(n.with_type_ref(q))));
                    }
                    None => {}
                }
                // try compact variable
                match v
                    .iter()
                    .find(|x| x.is_compact_variable_of_type(q_resolved))
                    .map(|x| x.qualifier())
                {
                    Some(q_declared) => {
                        return Ok(Some(Expression::Variable(n.with_type_ref(q_declared))));
                    }
                    None => {}
                }
            }
            None => {}
        }

        Ok(None)
    }

    fn resolve_expression_qualified(&self, n: &QualifiedName) -> Result<Option<Expression>, Error> {
        match self.names.get(n.as_ref()) {
            Some(v) => {
                // try extended parameters
                match v
                    .iter()
                    .find(|x| x.is_extended_param())
                    .map(|x| x.qualifier())
                {
                    Some(_) => {
                        return err_no_pos(LinterError::DuplicateDefinition);
                    }
                    None => {}
                }
                // try compact parameters
                match v
                    .iter()
                    .find(|x| x.is_compact_param_of_type(n.qualifier()))
                    .map(|x| x.qualifier())
                {
                    Some(q) => {
                        return Ok(Some(Expression::Variable(n.with_type_ref(q))));
                    }
                    None => {}
                }
                // try constants
                match v.iter().find(|x| x.is_constant()).map(|x| x.qualifier()) {
                    Some(q) => {
                        if n.is_of_type(q) {
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
                    .find(|x| x.is_extended_variable())
                    .map(|x| x.qualifier())
                {
                    Some(_) => {
                        return err_no_pos(LinterError::DuplicateDefinition);
                    }
                    None => {}
                }
                // try compact variable
                match v
                    .iter()
                    .find(|x| x.is_compact_variable_of_type(n.qualifier()))
                    .map(|x| x.qualifier())
                {
                    Some(q_declared) => {
                        return Ok(Some(Expression::Variable(n.with_type_ref(q_declared))));
                    }
                    None => {}
                }
            }
            None => {}
        }
        Ok(None)
    }

    fn resolve_parent_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        match self.names.get(n.as_ref()) {
            Some(v) => {
                for i in v {
                    match i {
                        Identifier::Constant(q) => {
                            if n.is_bare_or_of_type(*q) {
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

        self.resolve_parent_const_expression(n)
    }

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.function_name {
            Some(x) => x == name.as_ref(),
            None => false,
        }
    }

    pub fn has_param(&self, name: &Name) -> bool {
        match self.names.get(name.as_ref()) {
            Some(v) => v.iter().any(|x| x.is_param()),
            None => false,
        }
    }
}
