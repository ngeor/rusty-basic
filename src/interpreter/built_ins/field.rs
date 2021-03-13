use super::*;
use crate::common::*;
use crate::interpreter::io::Field;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let len = interpreter.context().variables().len();
    let file_handle: FileHandle = interpreter.context()[0].try_ref_into()?;
    let mut i: usize = 1;
    let mut fields: Vec<Field> = vec![];
    while i < len {
        let width: i32 = interpreter.context()[i].try_ref_into()?;
        i += 1;
        // TODO would be great to have a pointer to a variable here, maybe revisit when implementing DEF SEG
        let name: &String = interpreter.context()[i].try_as_ref()?;
        i += 2; // skip over the actual variable
        fields.push(Field {
            width,
            name: name.clone(),
        });
    }
    interpreter
        .file_manager()
        .add_field_list(file_handle, fields)?;
    Ok(())
}
