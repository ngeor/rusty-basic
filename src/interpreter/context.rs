use super::Variant;
use crate::parser::{HasQualifier, QualifiedName};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Context {
    Root(HashMap<QualifiedName, Variant>),
    Sub(HashMap<QualifiedName, Variant>, Box<Context>),
    Function(HashMap<QualifiedName, Variant>, QualifiedName, Box<Context>),
}

impl Context {
    pub fn new() -> Context {
        Context::Root(HashMap::new())
    }

    pub fn get(&self, name: &QualifiedName) -> Option<&Variant> {
        match self {
            Context::Root(m) | Context::Function(m, _, _) | Context::Sub(m, _) => m.get(name),
        }
    }

    pub fn insert(&mut self, name: QualifiedName, value: Variant) -> Option<Variant> {
        if Variant::default_variant(name.qualifier()).is_same_type(&value) {
            match self {
                Context::Root(m) | Context::Function(m, _, _) | Context::Sub(m, _) => {
                    m.insert(name, value)
                }
            }
        } else {
            panic!(format!("Type mismatch {} {}", name, value))
        }
    }

    pub fn push_function(self, result_name: QualifiedName) -> Context {
        Context::Function(self.clone_variable_map(), result_name, Box::new(self))
    }

    pub fn push_sub(self) -> Context {
        Context::Sub(self.clone_variable_map(), Box::new(self))
    }

    pub fn pop(self) -> Context {
        match self {
            Context::Root(_) => panic!("Stack underflow"),
            Context::Function(_, _, parent) | Context::Sub(_, parent) => *parent,
        }
    }

    fn clone_variable_map(&self) -> HashMap<QualifiedName, Variant> {
        match self {
            Context::Root(m) | Context::Function(m, _, _) | Context::Sub(m, _) => m.clone(),
        }
    }
}
