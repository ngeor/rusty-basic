use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{FileHandle, QError};
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::variant::{QBNumberCast, Variant};
use std::convert::TryFrom;

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QError> {
    match f {
        BuiltInFunction::Chr => chr::run(interpreter),
        BuiltInFunction::Environ => environ_fn::run(interpreter),
        BuiltInFunction::Eof => eof::run(interpreter),
        BuiltInFunction::InStr => instr::run(interpreter),
        BuiltInFunction::LBound => lbound::run(interpreter),
        BuiltInFunction::Len => len::run(interpreter),
        BuiltInFunction::Mid => mid::run(interpreter),
        BuiltInFunction::Str => str_fn::run(interpreter),
        BuiltInFunction::UBound => ubound::run(interpreter),
        BuiltInFunction::Val => val::run(interpreter),
    }
}

pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QError> {
    match s {
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Field => field::run(interpreter),
        BuiltInSub::Get => get::run(interpreter),
        BuiltInSub::Input => input::run(interpreter),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter),
        BuiltInSub::LSet => lset::run(interpreter),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
        BuiltInSub::Put => put::run(interpreter),
        BuiltInSub::ViewPrint => crate::built_ins::view_print::interpreter::run(interpreter),
    }
}

fn to_file_handle(v: &Variant) -> Result<FileHandle, QError> {
    let i: i32 = v.try_cast()?;
    FileHandle::try_from(i)
}

fn get_record_number(v: &Variant) -> Result<usize, QError> {
    let record_number_as_long: i64 = v.try_cast()?;
    if record_number_as_long <= 0 {
        Err(QError::BadRecordNumber)
    } else {
        Ok(record_number_as_long as usize)
    }
}

mod chr;
mod close;
mod environ_fn;
mod environ_sub;
mod eof;
mod field;
mod get;
mod input;
mod instr;
mod kill;
mod lbound;
mod len;
mod line_input;
mod lset;
mod mid;
mod name;
mod open;
mod put;
mod str_fn;
mod ubound;
mod val;
