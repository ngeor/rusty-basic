use crate::common::{HasLocation, Location};
use crate::interpreter::{Interpreter, InterpreterError, Result, Stdlib, Variant};
use crate::parser::{NameNode, QualifiedName, ResolveIntoRef};

pub trait VariableGetter {
    fn get_variable_at(&self, name: &QualifiedName, pos: Location) -> Result<&Variant>;
    fn get_variable(&self, name: &NameNode) -> Result<&Variant>;
}

impl<S: Stdlib> VariableGetter for Interpreter<S> {
    fn get_variable_at(&self, name: &QualifiedName, pos: Location) -> Result<&Variant> {
        match self.context_ref().get(name) {
            Some(v) => Ok(v),
            None => Err(InterpreterError::new_with_pos(
                format!("Variable {} not defined", name),
                pos,
            )),
        }
    }

    fn get_variable(&self, name: &NameNode) -> Result<&Variant> {
        let pos = name.location();
        let qualified_name: QualifiedName = name.resolve_into(self);
        self.get_variable_at(&qualified_name, pos)
    }
}
