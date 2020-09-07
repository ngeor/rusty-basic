use crate::common::*;
use crate::linter::type_resolver::*;
use crate::linter::types::{Expression, ResolvedDeclaredName, ResolvedTypeDefinition};
use crate::parser::{
    BareName, DeclaredName, Name, QualifiedName, TypeQualifier, WithTypeQualifier,
};
use std::collections::HashMap;

// ========================================================
// ResolvedTypeDefinitions
// ========================================================

#[derive(Debug, Default)]
struct ResolvedTypeDefinitions {
    v: Vec<ResolvedTypeDefinition>,
}

impl ResolvedTypeDefinitions {
    pub fn push(&mut self, t: ResolvedTypeDefinition) -> Result<(), QError> {
        if self.clashes_with(&t) {
            Err(QError::DuplicateDefinition)
        } else {
            self.v.push(t);
            Ok(())
        }
    }

    pub fn clashes_with(&self, t: &ResolvedTypeDefinition) -> bool {
        match &t {
            ResolvedTypeDefinition::CompactBuiltIn(q) => self
                .v
                .iter()
                .any(|x| x.is_extended() || x.is_compact_of_type(*q)),
            ResolvedTypeDefinition::ExtendedBuiltIn(_) | ResolvedTypeDefinition::UserDefined(_) => {
                !self.v.is_empty()
            }
        }
    }

    pub fn iter(&self) -> std::slice::Iter<ResolvedTypeDefinition> {
        self.v.iter()
    }

    pub fn opt_q<F>(&self, predicate: F) -> Option<TypeQualifier>
    where
        F: Fn(&ResolvedTypeDefinition) -> bool,
    {
        for resolved_type_definition in self.v.iter() {
            if predicate(resolved_type_definition) {
                match resolved_type_definition {
                    ResolvedTypeDefinition::CompactBuiltIn(q)
                    | ResolvedTypeDefinition::ExtendedBuiltIn(q) => {
                        return Some(*q);
                    }
                    ResolvedTypeDefinition::UserDefined(_) => {}
                }
            }
        }

        None
    }
}

//
// VariableMap
//

#[derive(Debug, Default)]
struct VariableMap {
    m: HashMap<BareName, ResolvedTypeDefinitions>,
}

impl VariableMap {
    pub fn push(&mut self, declared_name: ResolvedDeclaredName) -> Result<(), QError> {
        let ResolvedDeclaredName {
            name,
            type_definition,
        } = declared_name;
        match self.m.get_mut(&name) {
            Some(type_definitions) => type_definitions.push(type_definition),
            None => {
                let mut type_definitions = ResolvedTypeDefinitions::default();
                type_definitions.push(type_definition)?;
                self.m.insert(name, type_definitions);
                Ok(())
            }
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.m.contains_key(bare_name.as_ref())
    }

    pub fn clashes_with(&self, declared_name: &ResolvedDeclaredName) -> bool {
        let ResolvedDeclaredName {
            name,
            type_definition,
        } = declared_name;
        match self.m.get(name) {
            Some(type_definitions) => type_definitions.clashes_with(type_definition),
            None => false,
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<ResolvedDeclaredName>, QError> {
        let bare_name: &BareName = name.as_ref();
        let q: TypeQualifier = name.resolve_into(resolver);
        match self.m.get(bare_name) {
            Some(type_definitions) => {
                for type_definition in type_definitions.iter() {
                    if type_definition.is_extended() {
                        // only bare name is allowed
                        if name.is_bare() {
                            return Ok(Some(ResolvedDeclaredName {
                                name: bare_name.clone(),
                                type_definition: type_definition.clone(),
                            }));
                        } else {
                            return Err(QError::DuplicateDefinition);
                        }
                    } else if type_definition.is_compact_of_type(q) {
                        return Ok(Some(ResolvedDeclaredName {
                            name: bare_name.clone(),
                            type_definition: type_definition.clone(),
                        }));
                    }
                }
            }
            None => {}
        }
        Ok(None)
    }

    pub fn resolve_const_expression(&self, name: &Name) -> Result<Option<Expression>, QError> {
        match name {
            Name::Bare(b) => match self.m.get(b) {
                Some(type_definitions) => {
                    match type_definitions.opt_q(ResolvedTypeDefinition::is_compact_built_in) {
                        Some(q) => Ok(Some(Expression::Constant(b.with_type_ref(q)))),
                        None => Ok(None),
                    }
                }
                None => Ok(None),
            },
            Name::Qualified {
                name: bare_name,
                qualifier,
            } => match self.m.get(bare_name) {
                Some(type_definitions) => {
                    match type_definitions.opt_q(ResolvedTypeDefinition::is_compact_built_in) {
                        Some(q) => {
                            if q == *qualifier {
                                Ok(Some(Expression::Constant(name.with_type_ref(q))))
                            } else {
                                Err(QError::DuplicateDefinition)
                            }
                        }
                        None => Ok(None),
                    }
                }
                None => Ok(None),
            },
        }
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        match self.m.get(bare_name) {
            Some(type_definitions) => {
                match name {
                    Name::Bare(_) => {
                        // bare name can match extended identifiers
                        match type_definitions.opt_q(ResolvedTypeDefinition::is_extended) {
                            Some(q) => Ok(Some(Expression::Variable(bare_name.with_type_ref(q)))),
                            None => {
                                // let's try the resolver then
                                let q: TypeQualifier = bare_name.resolve_into(resolver);
                                if type_definitions.iter().any(|x| x.is_compact_of_type(q)) {
                                    Ok(Some(Expression::Variable(bare_name.with_type_ref(q))))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                    Name::Qualified { qualifier: q, .. } => {
                        // qualified names cannot match extended identifiers
                        match type_definitions.iter().find(|x| x.is_extended()) {
                            Some(_) => Err(QError::DuplicateDefinition),
                            None => {
                                if type_definitions.iter().any(|x| x.is_compact_of_type(*q)) {
                                    Ok(Some(Expression::Variable(bare_name.with_type_ref(*q))))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    }
                }
            }
            None => Ok(None),
        }
    }
}

//
// Symbols
//

#[derive(Debug, Default)]
pub struct Symbols {
    constants: VariableMap, // NOTE: constants can only be compact and the type resolution comes from the expression side if bare
    params: VariableMap,
    variables: VariableMap,
}

impl Symbols {
    fn resolve_declared_name<T: TypeResolver>(
        d: DeclaredName,
        resolver: &T,
    ) -> ResolvedDeclaredName {
        if d.is_bare() {
            let DeclaredName { name, .. } = d;
            let q: TypeQualifier = (&name).resolve_into(resolver);
            ResolvedDeclaredName {
                name,
                type_definition: ResolvedTypeDefinition::CompactBuiltIn(q),
            }
        } else {
            let DeclaredName {
                name,
                type_definition,
            } = d;
            ResolvedDeclaredName {
                name,
                type_definition: type_definition.into(),
            }
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.constants.contains_any(bare_name)
            || self.params.contains_any(bare_name)
            || self.variables.contains_any(bare_name)
    }

    pub fn push_param<T: TypeResolver>(
        &mut self,
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<(), QError> {
        self.params
            .push(Self::resolve_declared_name(declared_name, resolver))
    }

    pub fn push_const(&mut self, q_name: QualifiedName) -> Result<(), QError> {
        if self.contains_any(&q_name) {
            Err(QError::DuplicateDefinition)
        } else {
            let QualifiedName { name, qualifier } = q_name;
            self.constants.push(ResolvedDeclaredName {
                name,
                type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
            })
        }
    }

    pub fn push_dim<T: TypeResolver>(
        &mut self,
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<ResolvedDeclaredName, QError> {
        let r = Self::resolve_declared_name(declared_name, resolver);
        if self.constants.contains_any(&r)
            || self.params.clashes_with(&r)
            || self.variables.clashes_with(&r)
        {
            Err(QError::DuplicateDefinition)
        } else {
            self.variables.push(r.clone())?;
            Ok(r)
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<ResolvedDeclaredName>, QError> {
        // first params
        // then constants
        // then variables

        if self.constants.contains_any(name) {
            Err(QError::DuplicateDefinition)
        } else {
            self.params
                .resolve_assignment(name, resolver)
                .or_try_read(|| self.variables.resolve_assignment(name, resolver))
        }
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, QError> {
        // is it param
        // is it constant
        // is it variable
        // is it parent constant
        // it's a new implicit variable

        self.params
            .resolve_expression(name, resolver)
            .or_try_read(|| self.resolve_const_expression(name))
            .or_try_read(|| self.variables.resolve_expression(name, resolver))
    }

    pub fn resolve_const_expression(&self, name: &Name) -> Result<Option<Expression>, QError> {
        self.constants.resolve_const_expression(name)
    }

    pub fn resolve_param_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<bool, QError> {
        self.params
            .resolve_assignment(name, resolver)
            .map(|x| x.is_some())
    }

    pub fn resolve_missing_name_in_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<ResolvedDeclaredName, QError> {
        let QualifiedName { name, qualifier } = name.resolve_into(resolver);
        let d = ResolvedDeclaredName {
            name,
            type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
        };
        self.variables.push(d.clone())?;
        Ok(d)
    }

    pub fn resolve_missing_name_in_expression<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Expression, QError> {
        let QualifiedName { name, qualifier } = name.resolve_into(resolver);
        let d = ResolvedDeclaredName {
            name: name.clone(),
            type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
        };
        self.variables.push(d)?;
        Ok(Expression::Variable(QualifiedName { name, qualifier }))
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

#[derive(Debug, Default)]
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    sub_program: Option<(CaseInsensitiveString, SubProgramType)>,
    symbols: Symbols,
}

impl LinterContext {
    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }

    pub fn symbols_mut(&mut self) -> &mut Symbols {
        &mut self.symbols
    }

    pub fn push_function_context(self, name: &CaseInsensitiveString) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.sub_program = Some((name.clone(), SubProgramType::Function));
        result
    }

    pub fn push_sub_context(self, name: &CaseInsensitiveString) -> Self {
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
        self.symbols.push_const(q_name.clone()).with_err_no_pos()?;
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
        self.symbols
            .resolve_expression(n, resolver)
            .or_try_read(|| self.resolve_parent_const_expression(n))
    }

    fn resolve_parent_const_expression(&self, n: &Name) -> Result<Option<Expression>, QError> {
        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    pub fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, QError> {
        match self.symbols.resolve_const_expression(n)? {
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
        match self.symbols.resolve_assignment(name, resolver)? {
            Some(x) => Ok(x),
            None => {
                // maybe a parent constant?
                match self.resolve_parent_const_expression(name)? {
                    Some(_) => Err(QError::DuplicateDefinition),
                    None => {
                        // just insert it
                        self.symbols
                            .resolve_missing_name_in_assignment(name, resolver)
                    }
                }
            }
        }
    }
}
