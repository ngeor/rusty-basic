use crate::common::Result;
use crate::parser::{QName, TypeQualifier};
use std::collections::HashMap;
use super::Variant;

/// A variable context
#[derive(Debug, Clone)]
pub struct Context {
    variable_map: HashMap<(String, TypeQualifier), Variant>,
}

pub trait ReadOnlyContext {
    fn get_variable(&self, variable_name: &QName) -> Result<Variant>;
}

pub trait ReadWriteContext: ReadOnlyContext {
    fn set_variable(&mut self, variable_name: &QName, variable_value: Variant) -> Result<()>;
}

impl Context {
    pub fn new() -> Context {
        Context {
            variable_map: HashMap::new(),
        }
    }
}

fn _to_tuple(variable_name: &QName) -> Result<(String, TypeQualifier)> {
    match variable_name {
        QName::Untyped(_) => Err("Unexpected untyped variable".to_string()),
        QName::Typed(name, qualifier) => Ok((name.clone(), qualifier.clone())),
    }
}

impl ReadOnlyContext for Context {
    fn get_variable(&self, variable_name: &QName) -> Result<Variant> {
        let t = _to_tuple(variable_name)?;
        match self.variable_map.get(&t) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("Variable {} is not defined", variable_name)),
        }
    }
}

impl ReadWriteContext for Context {
    fn set_variable(&mut self, variable_name: &QName, variable_value: Variant) -> Result<()> {
        let t = _to_tuple(variable_name)?;
        self.variable_map.insert(t, variable_value);
        Ok(())
    }
}
