use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{FileAccess, FileHandle, FileMode, QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::instruction_generator::InternalBuiltInSub;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Field;
use crate::interpreter::stdlib::Stdlib;
use crate::parser::{BareName, ElementType, TypeQualifier, UserDefinedType, UserDefinedTypes};
use crate::variant::{Variant, MAX_INTEGER, MAX_LONG};
use std::convert::{TryFrom, TryInto};

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QErrorNode> {
    match f {
        BuiltInFunction::Chr => chr::run(interpreter),
        BuiltInFunction::Environ => environ_fn::run(interpreter),
        BuiltInFunction::Eof => eof::run(interpreter).with_err_no_pos(),
        BuiltInFunction::InStr => instr::run(interpreter),
        BuiltInFunction::LBound => lbound::run(interpreter),
        BuiltInFunction::Len => len::run(interpreter),
        BuiltInFunction::Mid => mid::run(interpreter),
        BuiltInFunction::Str => str_fn::run(interpreter),
        BuiltInFunction::UBound => ubound::run(interpreter),
        BuiltInFunction::Val => val::run(interpreter),
    }
}

pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QErrorNode> {
    match s {
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Input => input::run(interpreter).with_err_no_pos(),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter).with_err_no_pos(),
        BuiltInSub::LSet => lset::run(interpreter).with_err_no_pos(),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
    }
}

mod lset {
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let name = String::try_from(interpreter.context()[0].clone())?;
        let value = interpreter.context()[2].clone();
        // find which file number is associated with this name and find the width
        // also marks that field index as current for the next PUT operation
        interpreter.file_manager().lookup_width(&name)?;
        interpreter.context_mut()[1] = value;
        Ok(())
    }
}

pub fn rub_internal_built_in_sub<S: InterpreterTrait>(
    s: &InternalBuiltInSub,
    interpreter: &mut S,
) -> Result<(), QErrorNode> {
    match s {
        InternalBuiltInSub::Field => {
            let len = interpreter.context().variables().len();
            let file_handle = FileHandle::try_from(&interpreter.context()[0]).with_err_no_pos()?;
            let mut i: usize = 1;
            let mut fields: Vec<Field> = vec![];
            while i < len {
                let width = i32::try_from(&interpreter.context()[i]).with_err_no_pos()?;
                i += 1;
                // TODO would be great to have a pointer to a variable here, maybe revisit when implementing DEF SEG
                let name = String::try_from(interpreter.context()[i].clone()).with_err_no_pos()?;
                i += 1;
                fields.push(Field { width, name });
            }
            interpreter
                .file_manager()
                .add_field_list(file_handle, fields)
                .with_err_no_pos()?;
            Ok(())
        }
        InternalBuiltInSub::Get => {
            let handle = FileHandle::try_from(&interpreter.context()[0]).with_err_no_pos()?;
            let record_number = i32::try_from(&interpreter.context()[1]).with_err_no_pos()?;
            interpreter.pop_context();
            let file_info = interpreter
                .file_manager()
                .try_get_file_info_mut(&handle)
                .with_err_no_pos()?;
            let field_lists: Vec<Vec<Field>> = file_info.get_field_lists().clone(); // fighting the borrow checker
            let bytes = file_info
                .get_record(record_number as usize)
                .with_err_no_pos()?;
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
                    // set variable
                    interpreter.context_mut().variables_mut().insert_built_in(
                        BareName::from(name),
                        TypeQualifier::DollarString,
                        v,
                    );
                    // shift to next offset
                    start += width_usize;
                }
            }
            Ok(())
        }
        InternalBuiltInSub::Put => {
            let handle = FileHandle::try_from(&interpreter.context()[0]).with_err_no_pos()?;
            let record_number = i32::try_from(&interpreter.context()[1]).with_err_no_pos()?;
            interpreter.pop_context();
            let file_info = interpreter
                .file_manager()
                .try_get_file_info_mut(&handle)
                .with_err_no_pos()?;
            let mut record_contents = String::new();
            // get the current field list
            let field_list = file_info
                .get_current_field_list()
                .ok_or(QError::BadFileMode)
                .with_err_no_pos()?
                .clone();
            // convert the variables into a string
            for Field { width, name } in field_list {
                let width_usize: usize = width as usize;
                let v = interpreter
                    .context()
                    .variables()
                    .get_built_in(&BareName::from(name.as_str()), TypeQualifier::DollarString)
                    .ok_or(QError::VariableRequired)
                    .with_err_no_pos()?;
                if let Variant::VString(v_s) = v {
                    for i in 0..width_usize {
                        if i < v_s.len() {
                            record_contents
                                .push(v_s.chars().skip(i).take(1).next().unwrap() as char);
                        } else {
                            record_contents.push('\0');
                        }
                    }
                } else {
                    return Err(QError::VariableRequired).with_err_no_pos();
                }
            }
            let file_info = interpreter
                .file_manager()
                .try_get_file_info_mut(&handle)
                .with_err_no_pos()?;
            file_info
                .put_record(record_number as usize, record_contents.as_bytes())
                .with_err_no_pos()?;
            Ok(())
        }
    }
}

mod chr;
mod close;
mod environ_fn;
mod environ_sub;
mod eof;
mod input;
mod instr;
mod kill;
mod lbound;
mod len;
mod line_input;
mod mid;
mod name;
mod open;
mod str_fn;
mod ubound;
mod val;
