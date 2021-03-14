use super::*;
use crate::common::*;
use crate::interpreter::io::Field;
use crate::variant::{QBNumberCast, Variant};

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let len = interpreter.context().variables().len();
    let file_handle: FileHandle = to_file_handle(&interpreter.context()[0])?;
    let mut i: usize = 1;
    let mut fields: Vec<Field> = vec![];
    while i < len {
        let width: usize = get_field_width(&interpreter.context()[i])?;
        i += 1;
        // TODO would be great to have a pointer to a variable here, maybe revisit when implementing DEF SEG
        let name: &str = interpreter.context()[i].to_str_unchecked();
        i += 2; // skip over the actual variable
        fields.push(Field {
            width,
            name: name.to_owned(),
        });
    }
    interpreter
        .file_manager()
        .add_field_list(file_handle, fields)?;
    Ok(())
}

fn get_field_width(v: &Variant) -> Result<usize, QError> {
    let field_width_as_integer: i32 = v.try_cast()?;
    if field_width_as_integer <= 0 {
        Err(QError::FieldOverflow)
    } else {
        Ok(field_width_as_integer as usize)
    }
}
