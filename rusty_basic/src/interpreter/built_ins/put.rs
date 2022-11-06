use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Field;
use crate::interpreter::string_utils::to_ascii_bytes;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;
use rusty_parser::{BareName, FileHandle, TypeQualifier};

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let handle: FileHandle = interpreter.context()[0].to_file_handle()?;
    let record_number: usize = interpreter.context()[1].to_record_number()?;
    let file_info = interpreter.file_manager().try_get_file_info(&handle)?;
    let mut record_contents: Vec<u8> = vec![];
    // get the current field list
    let field_list = file_info
        .get_current_field_list()
        .ok_or(RuntimeError::BadFileMode)?
        .clone(); // TODO fighting the borrow checker
                  // convert the variables into a string
    for Field { width, name } in field_list {
        let bare_name: BareName = BareName::from(name.as_str());
        let v = interpreter
            .context()
            .caller_variables()
            .get_built_in(&bare_name, TypeQualifier::DollarString)
            .ok_or(RuntimeError::VariableRequired)?;
        let mut bytes: Vec<u8> = to_ascii_bytes(v.to_str_unchecked());
        fix_length(&mut bytes, width);
        record_contents.append(&mut bytes);
    }
    let file_info = interpreter.file_manager().try_get_file_info(&handle)?;
    file_info.put_record(record_number, &record_contents)?;
    Ok(())
}

fn fix_length(bytes: &mut Vec<u8>, width: usize) {
    while bytes.len() < width {
        bytes.push(0);
    }
    while bytes.len() > width {
        bytes.pop();
    }
}
