use super::*;
use crate::variant::Variant;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let name: String = interpreter.context()[0].to_str_unchecked().to_owned(); // TODO fighting borrow checker
    let value: Variant = interpreter.context()[2].clone();
    // find which file number is associated with this name and find the width
    // also marks that field index as current for the next PUT operation
    interpreter.file_manager().lookup_width(&name)?;
    interpreter.context_mut()[1] = value;
    Ok(())
}
