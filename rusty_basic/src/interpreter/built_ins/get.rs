use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Field;
use crate::interpreter::string_utils::to_ascii_string;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;
use rusty_parser::{BareName, FileHandle, TypeQualifier};
use rusty_variant::Variant;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let handle: FileHandle = interpreter.context()[0].to_file_handle()?;
    let record_number: usize = interpreter.context()[1].to_record_number()?;
    let file_info = interpreter.file_manager().try_get_file_info(&handle)?;
    let field_lists: Vec<Vec<Field>> = file_info.get_field_lists().clone(); // TODO fighting the borrow checker
    let bytes = file_info.get_record(record_number)?;
    for fields in field_lists {
        let mut start: usize = 0;
        for Field { width, name } in fields {
            let s = to_ascii_string(&bytes[start..(start + width)]);
            let v = Variant::VString(s);
            // set variable in parent context, because we're inside the context of the built-in sub
            let bare_name: BareName = BareName::from(name);
            interpreter
                .context_mut()
                .caller_variables_mut()
                .insert_built_in(bare_name, TypeQualifier::DollarString, v);
            // shift to next offset
            start += width;
        }
    }
    Ok(())
}
