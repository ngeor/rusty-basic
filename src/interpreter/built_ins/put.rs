use super::*;
use crate::common::{FileHandle, TryRefInto};
use crate::interpreter::io::Field;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::Variant;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let handle: FileHandle = interpreter.context()[0].try_ref_into()?;
    let record_number = i32::try_from(&interpreter.context()[1])?;
    let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
    let mut record_contents = String::new();
    // get the current field list
    let field_list = file_info
        .get_current_field_list()
        .ok_or(QError::BadFileMode)?
        .clone();
    // convert the variables into a string
    for Field { width, name } in field_list {
        let width_usize: usize = width as usize;
        let bare_name: BareName = BareName::from(name.as_str());
        let v = interpreter
            .context()
            .caller_variables()
            .get_built_in(&bare_name, TypeQualifier::DollarString)
            .ok_or(QError::VariableRequired)?;
        if let Variant::VString(v_s) = v {
            for i in 0..width_usize {
                if i < v_s.len() {
                    record_contents.push(v_s.chars().skip(i).take(1).next().unwrap() as char);
                } else {
                    record_contents.push('\0');
                }
            }
        } else {
            return Err(QError::VariableRequired);
        }
    }
    let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
    file_info.put_record(record_number as usize, record_contents.as_bytes())?;
    Ok(())
}
