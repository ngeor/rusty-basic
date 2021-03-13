use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

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
