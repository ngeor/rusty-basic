use crate::common::*;
use crate::linter::error::*;
use crate::linter::type_resolver::*;
use crate::linter::Expression;
use crate::parser::{
    BareName, DeclaredName, HasQualifier, Name, QualifiedName, TypeDefinition, TypeQualifier,
    WithTypeQualifier,
};
use std::collections::HashMap;
use std::convert::TryFrom;

//
// TypeDefinitions
//

#[derive(Debug, Default)]
struct TypeDefinitions {
    v: Vec<TypeDefinition>,
}

impl TypeDefinitions {
    pub fn push(&mut self, t: TypeDefinition) -> Result<(), Error> {
        if self.clashes_with(&t) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            self.v.push(t);
            Ok(())
        }
    }

    pub fn clashes_with(&self, t: &TypeDefinition) -> bool {
        match &t {
            TypeDefinition::Bare => {
                panic!("Internal error: unresolved bare type definition");
            }
            TypeDefinition::CompactBuiltIn(q) => self
                .v
                .iter()
                .any(|x| x.is_extended() || x.is_compact_of_type(*q)),
            TypeDefinition::ExtendedBuiltIn(_) | TypeDefinition::UserDefined(_) => {
                !self.v.is_empty()
            }
        }
    }

    pub fn iter(&self) -> std::slice::Iter<TypeDefinition> {
        self.v.iter()
    }
}

//
// VariableMap
//

#[derive(Debug, Default)]
struct VariableMap {
    m: HashMap<BareName, TypeDefinitions>,
}

impl VariableMap {
    pub fn push(&mut self, declared_name: DeclaredName) -> Result<(), Error> {
        // TODO use the destructuring pattern elsewhere too
        let DeclaredName {
            name,
            type_definition,
        } = declared_name;
        match self.m.get_mut(&name) {
            Some(type_definitions) => type_definitions.push(type_definition),
            None => {
                let mut type_definitions = TypeDefinitions::default();
                type_definitions.push(type_definition)?;
                self.m.insert(name, type_definitions);
                Ok(())
            }
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: &T) -> bool {
        self.m.contains_key(bare_name.as_ref())
    }

    pub fn clashes_with(&self, declared_name: &DeclaredName) -> bool {
        let bare_name: &BareName = declared_name.as_ref();
        match self.m.get(bare_name) {
            Some(type_definitions) => {
                type_definitions.clashes_with(declared_name.type_definition())
            }
            None => false,
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<DeclaredName>, Error> {
        let bare_name: &BareName = name.as_ref();
        let q: TypeQualifier = name.resolve_into(resolver);
        match self.m.get(bare_name) {
            Some(type_definitions) => {
                for type_definition in type_definitions.iter() {
                    if type_definition.is_extended() {
                        // only bare name is allowed
                        if name.is_bare() {
                            return Ok(Some(DeclaredName::new(
                                bare_name.clone(),
                                type_definition.clone(),
                            )));
                        } else {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        }
                    } else if type_definition.is_compact_of_type(q) {
                        return Ok(Some(DeclaredName::new(
                            bare_name.clone(),
                            type_definition.clone(),
                        )));
                    }
                }
            }
            None => {}
        }
        Ok(None)
    }

    pub fn resolve_const_expression(&self, name: &Name) -> Result<Option<Expression>, Error> {
        match name {
            Name::Bare(b) => match self.m.get(b) {
                Some(type_definitions) => {
                    let opt_q = type_definitions
                        .iter()
                        .find(|x| x.is_compact_built_in())
                        .map(|x| TypeQualifier::try_from(x).unwrap());

                    match opt_q {
                        Some(q) => Ok(Some(Expression::Constant(b.with_type_ref(q)))),
                        None => Ok(None),
                    }
                }
                None => Ok(None),
            },
            Name::Qualified(q_name) => match self.m.get(q_name.as_ref()) {
                Some(type_definitions) => {
                    let opt_q = type_definitions
                        .iter()
                        .find(|x| x.is_compact_built_in())
                        .map(|x| TypeQualifier::try_from(x).unwrap());

                    match opt_q {
                        Some(q) => {
                            if q == q_name.qualifier() {
                                Ok(Some(Expression::Constant(q_name.clone())))
                            } else {
                                err_no_pos(LinterError::DuplicateDefinition)
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
    ) -> Result<Option<Expression>, Error> {
        let bare_name: &BareName = name.as_ref();
        match self.m.get(bare_name) {
            Some(type_definitions) => {
                match name {
                    Name::Bare(_) => {
                        // bare name can match extended identifiers
                        match type_definitions
                            .iter()
                            .find(|x| x.is_extended())
                            .map(|x| TypeQualifier::try_from(x).unwrap())
                        {
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
                    Name::Qualified(q_name) => {
                        // qualified names cannot match extended identifiers
                        match type_definitions.iter().find(|x| x.is_extended()) {
                            Some(_) => err_no_pos(LinterError::DuplicateDefinition),
                            None => {
                                let q: TypeQualifier = q_name.qualifier();
                                if type_definitions.iter().any(|x| x.is_compact_of_type(q)) {
                                    Ok(Some(Expression::Variable(q_name.clone())))
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
struct Symbols {
    constants: VariableMap, // NOTE: constants can only be compact and the type resolution comes from the expression side if bare
    params: VariableMap,
    variables: VariableMap,
}

impl Symbols {
    fn resolve_declared_name<T: TypeResolver>(d: DeclaredName, resolver: &T) -> DeclaredName {
        if d.is_bare() {
            let DeclaredName { name, .. } = d;
            let q: TypeQualifier = (&name).resolve_into(resolver);
            DeclaredName::new(name, TypeDefinition::CompactBuiltIn(q))
        } else {
            d
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
    ) -> Result<(), Error> {
        self.params
            .push(Self::resolve_declared_name(declared_name, resolver))
    }

    pub fn push_const(&mut self, q_name: QualifiedName) -> Result<(), Error> {
        if self.contains_any(&q_name) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            let QualifiedName { name, qualifier } = q_name;
            self.constants.push(DeclaredName::new(
                name,
                TypeDefinition::CompactBuiltIn(qualifier),
            ))
        }
    }

    pub fn push_dim<T: TypeResolver>(
        &mut self,
        declared_name: DeclaredName,
        resolver: &T,
    ) -> Result<DeclaredName, Error> {
        let r = Self::resolve_declared_name(declared_name, resolver);
        if self.constants.contains_any(&r)
            || self.params.clashes_with(&r)
            || self.variables.clashes_with(&r)
        {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            self.variables.push(r.clone())?;
            Ok(r)
        }
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DeclaredName, Error> {
        // first params
        // then constants
        // then variables
        // then parent constants todo

        if self.constants.contains_any(name) {
            err_no_pos(LinterError::DuplicateDefinition)
        } else {
            self.params
                .resolve_assignment(name, resolver)
                .or_try_read(|| self.variables.resolve_assignment(name, resolver))
                .or_read(|| {
                    let q: QualifiedName = name.resolve_into(resolver);
                    let d = DeclaredName::from(q);
                    self.variables.push(d.clone())?;
                    Ok(d)
                })
        }
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, Error> {
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

    pub fn resolve_const_expression(&self, name: &Name) -> Result<Option<Expression>, Error> {
        self.constants.resolve_const_expression(name)
    }

    pub fn resolve_param_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<bool, Error> {
        self.params
            .resolve_assignment(name, resolver)
            .map(|x| x.is_some())
    }

    pub fn resolve_missing_variable<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Expression, Error> {
        let q: QualifiedName = name.resolve_into(resolver);
        let d = DeclaredName::from(q.clone());
        self.variables.push(d)?;
        Ok(Expression::Variable(q))
    }
}

//
// LinterContext
//

#[derive(Debug, Default)]
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    // TODO replace with one sub_program_name
    function_name: Option<CaseInsensitiveString>,
    sub_name: Option<CaseInsensitiveString>,
    symbols: Symbols,
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

    pub fn push_param<T: TypeResolver>(
        &mut self,
        name: DeclaredName,
        resolver: &T,
    ) -> Result<(), Error> {
        self.symbols.push_param(name, resolver)
    }

    pub fn push_const(
        &mut self,
        name: Name,
        right_side_type: Locatable<TypeQualifier>,
    ) -> Result<QualifiedName, Error> {
        let q = match &name {
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
        let q_name = name.with_type(q);
        self.symbols.push_const(q_name.clone())?;
        Ok(q_name)
    }

    // e.g. DIM A, DIM A$, DIM AS STRING
    pub fn push_dim<T: TypeResolver>(
        &mut self,
        name: DeclaredName,
        resolver: &T,
    ) -> Result<DeclaredName, Error> {
        self.symbols.push_dim(name.clone(), resolver)
    }

    pub fn resolve_expression<T: TypeResolver>(
        &self,
        n: &Name,
        resolver: &T,
    ) -> Result<Option<Expression>, Error> {
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

    fn resolve_parent_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        // try parent constants
        match &self.parent {
            Some(p) => p.resolve_const_expression(n),
            None => Ok(None),
        }
    }

    fn resolve_const_expression(&self, n: &Name) -> Result<Option<Expression>, Error> {
        match self.symbols.resolve_const_expression(n)? {
            Some(e) => Ok(Some(e)),
            None => self.resolve_parent_const_expression(n),
        }
    }

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.function_name {
            Some(x) => x == name.as_ref(),
            None => false,
        }
    }

    pub fn resolve_param_assignment<T: TypeResolver>(
        &self,
        name: &Name,
        resolver: &T,
    ) -> Result<bool, Error> {
        self.symbols.resolve_param_assignment(name, resolver)
    }

    pub fn resolve_assignment<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<DeclaredName, Error> {
        self.symbols.resolve_assignment(name, resolver)
    }

    pub fn resolve_missing_variable<T: TypeResolver>(
        &mut self,
        name: &Name,
        resolver: &T,
    ) -> Result<Expression, Error> {
        self.symbols.resolve_missing_variable(name, resolver)
    }
}
