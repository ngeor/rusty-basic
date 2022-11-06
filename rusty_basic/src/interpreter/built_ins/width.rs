use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), RuntimeError> {
    Ok(())
}
