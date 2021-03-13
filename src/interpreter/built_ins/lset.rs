use super::*;
use crate::common::QError;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let name = String::try_from(interpreter.context()[0].clone())?;
    let value = interpreter.context()[2].clone();
    // find which file number is associated with this name and find the width
    // also marks that field index as current for the next PUT operation
    interpreter.file_manager().lookup_width(&name)?;
    interpreter.context_mut()[1] = value;
    Ok(())
}
