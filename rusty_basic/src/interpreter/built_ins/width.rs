use crate::RuntimeError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), RuntimeError> {
    Ok(())
}
