use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Field;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;
use rusty_parser::specific::FileHandle;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let len = interpreter.context().variables().len();
    let file_handle: FileHandle = interpreter.context()[0].to_file_handle()?;
    let mut i: usize = 1;
    let mut fields: Vec<Field> = vec![];
    while i < len {
        let width: usize =
            interpreter.context()[i].to_positive_int_or(RuntimeError::FieldOverflow)?;
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
