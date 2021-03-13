use super::*;
use crate::common::{FileHandle, QError};
use crate::interpreter::io::Field;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let len = interpreter.context().variables().len();
    let file_handle: FileHandle = FileHandle::try_from(&interpreter.context()[0])?;
    let mut i: usize = 1;
    let mut fields: Vec<Field> = vec![];
    while i < len {
        let width = i32::try_from(&interpreter.context()[i])?;
        i += 1;
        // TODO would be great to have a pointer to a variable here, maybe revisit when implementing DEF SEG
        let name = String::try_from(interpreter.context()[i].clone())?;
        i += 2; // skip over the actual variable
        fields.push(Field { width, name });
    }
    interpreter
        .file_manager()
        .add_field_list(file_handle, fields)?;
    Ok(())
}
