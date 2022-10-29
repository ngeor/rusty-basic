use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;

pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QError> {
    Ok(())
}
