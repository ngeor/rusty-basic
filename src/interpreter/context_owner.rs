use super::{Interpreter, Stdlib};
use crate::interpreter::context::Context;

/// Represents the owner of a variable context.
pub trait ContextOwner {
    /// Pushes a new context as a result of a sub or function call.
    fn push_args_context(&mut self);

    fn swap_args_with_sub_context(&mut self);

    /// Pops a context.
    fn pop(&mut self);

    fn context_ref(&self) -> &Context;
    fn context_mut(&mut self) -> &mut Context;
}

impl<S: Stdlib> ContextOwner for Interpreter<S> {
    fn push_args_context(&mut self) {
        self.context = self.context.take().map(|x| x.push_args_context());
    }

    fn swap_args_with_sub_context(&mut self) {
        self.context = self.context.take().map(|x| x.swap_args_with_sub_context());
    }

    fn pop(&mut self) {
        self.context = self.context.take().map(|x| x.pop());
    }

    fn context_ref(&self) -> &Context {
        match &self.context {
            Some(x) => x,
            None => panic!("stack underflow"),
        }
    }

    fn context_mut(&mut self) -> &mut Context {
        match &mut self.context {
            Some(x) => x,
            None => panic!("stack underflow"),
        }
    }
}
