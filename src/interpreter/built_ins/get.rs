use super::*;
use crate::common::FileHandle;
use crate::interpreter::io::Field;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::Variant;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let handle: FileHandle = FileHandle::try_from(&interpreter.context()[0])?;
    let record_number = i32::try_from(&interpreter.context()[1])?;
    let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
    let field_lists: Vec<Vec<Field>> = file_info.get_field_lists().clone(); // fighting the borrow checker
    let bytes = file_info.get_record(record_number as usize)?;
    for fields in field_lists {
        let mut start: usize = 0;
        for Field { width, name } in fields {
            // collect ASCII chars stop at non printable char
            let width_usize: usize = width as usize;
            let mut s = String::new();
            for byte in &bytes[start..(start + width_usize)] {
                let ch = *byte as char;
                if ch >= ' ' {
                    s.push(ch);
                } else {
                    break;
                }
            }
            let v = Variant::VString(s);
            // set variable in parent context, because we're inside the context of the built-in sub
            let bare_name: BareName = BareName::from(name);
            interpreter
                .context_mut()
                .caller_variables_mut()
                .insert_built_in(bare_name, TypeQualifier::DollarString, v);
            // shift to next offset
            start += width_usize;
        }
    }
    Ok(())
}
