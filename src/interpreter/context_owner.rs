use super::{Interpreter, Result, Stdlib, Variant};
use crate::common::Location;
use crate::interpreter::variable_setter::VariableSetter;
use crate::parser::{Name, NameNode, QualifiedName};

/// Represents the owner of a variable context.
pub trait ContextOwner {
    /// Pushes a new context as a result of a sub call.
    fn push_sub(&mut self);

    /// Pushes a new context as a result of a function call.
    fn push_function(&mut self, result_name: QualifiedName);

    /// Pops a context.
    fn pop(&mut self);

    fn populate(
        &mut self,
        names: Vec<QualifiedName>,
        values: Vec<Variant>,
        call_pos: Location,
    ) -> Result<()>;
}

impl<S: Stdlib> ContextOwner for Interpreter<S> {
    fn push_function(&mut self, result_name: QualifiedName) {
        self.context = self.context.take().map(|x| x.push_function(result_name));
    }

    fn push_sub(&mut self) {
        self.context = self.context.take().map(|x| x.push_sub());
    }

    fn pop(&mut self) {
        self.context = self.context.take().map(|x| x.pop());
    }

    fn populate(
        &mut self,
        names: Vec<QualifiedName>,
        values: Vec<Variant>,
        call_pos: Location,
    ) -> Result<()> {
        for x in names.into_iter().zip(values.into_iter()) {
            let (qualified_name, value) = x;
            let name_node = NameNode::new(Name::Typed(qualified_name), call_pos);
            self.set_variable(name_node, value)?;
        }
        Ok(())
    }
}
