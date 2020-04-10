use super::casting::cast;
use super::{InterpreterError, Result, Variant};
use crate::common::{HasLocation, StripLocation};
use crate::parser::{BareNameNode, QualifiedName, QualifiedNameNode};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Context {
    Root(HashMap<QualifiedName, Variant>),
    Nested(HashMap<QualifiedName, Variant>, QualifiedName, Box<Context>),
}

impl Context {
    pub fn new() -> Context {
        Context::Root(HashMap::new())
    }

    pub fn get(&self, name: &QualifiedName) -> Option<&Variant> {
        match self {
            Context::Root(m) | Context::Nested(m, _, _) => m.get(name),
        }
    }

    pub fn insert(&mut self, name: QualifiedName, value: Variant) -> Option<Variant> {
        if Variant::default_variant(name.qualifier()).is_same_type(&value) {
            match self {
                Context::Root(m) | Context::Nested(m, _, _) => m.insert(name, value),
            }
        } else {
            panic!(format!("Type mismatch {} {}", name, value))
        }
    }

    pub fn push(self, result_name: QualifiedName) -> Context {
        Context::Nested(self.clone_variable_map(), result_name, Box::new(self))
    }

    pub fn pop(self) -> Context {
        match self {
            Context::Root(_) => panic!("Stack underflow"),
            Context::Nested(_, _, parent) => *parent,
        }
    }

    fn clone_variable_map(&self) -> HashMap<QualifiedName, Variant> {
        match self {
            Context::Root(m) | Context::Nested(m, _, _) => m.clone(),
        }
    }

    pub fn resolve_result_name_bare(&self, name: &BareNameNode) -> Option<QualifiedNameNode> {
        match self {
            Context::Root(_) => None,
            Context::Nested(_, result_name, _) => {
                if name.name() != result_name.name() {
                    // different names, it does not match with the result name
                    None
                } else {
                    // names match
                    // promote the bare name node to a qualified
                    Some(name.to_qualified_name_node(result_name.qualifier()))
                }
            }
        }
    }

    pub fn resolve_result_name_typed(&self, name: &QualifiedNameNode) -> Result<()> {
        match self {
            Context::Root(_) => Ok(()),
            Context::Nested(_, result_name, _) => {
                if name.name() != result_name.name() {
                    // different names, it does not match with the result name
                    Ok(())
                } else {
                    // names match
                    if name.qualifier() == result_name.qualifier() {
                        Ok(())
                    } else {
                        Err(InterpreterError::new_with_pos(
                            "Duplicate definition",
                            name.location(),
                        ))
                    }
                }
            }
        }
    }

    pub fn cast_insert(
        &mut self,
        name: &QualifiedNameNode,
        value: Variant,
    ) -> Result<Option<Variant>> {
        cast(value, name.qualifier())
            .map_err(|e| InterpreterError::new_with_pos(e, name.location()))
            .map(|casted| self.insert(name.strip_location(), casted))
    }
}
