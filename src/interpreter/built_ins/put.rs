use super::*;
use crate::common::{FileHandle, StringUtils};
use crate::interpreter::io::Field;
use crate::parser::{BareName, TypeQualifier};

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let handle: FileHandle = to_file_handle(&interpreter.context()[0])?;
    let record_number: usize = get_record_number(&interpreter.context()[1])?;
    let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
    let mut record_contents = String::new();
    // get the current field list
    let field_list = file_info
        .get_current_field_list()
        .ok_or(QError::BadFileMode)?
        .clone();
    // convert the variables into a string
    for Field { width, name } in field_list {
        let bare_name: BareName = BareName::from(name.as_str());
        let v = interpreter
            .context()
            .caller_variables()
            .get_built_in(&bare_name, TypeQualifier::DollarString)
            .ok_or(QError::VariableRequired)?;
        let s = v.to_str_unchecked().fix_length_with_char(width, '\0');
        record_contents.push_str(s.as_str());
    }
    let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
    file_info.put_record(record_number, record_contents.as_bytes())?;
    Ok(())
}