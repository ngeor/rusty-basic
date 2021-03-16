use crate::built_ins::BuiltInFunction;
use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QError> {
    match f {
        BuiltInFunction::Chr => crate::built_ins::chr::interpreter::run(interpreter),
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

mod environ_fn;
mod eof;
mod instr;
mod lbound;
mod len;
mod mid;
mod str_fn;
mod ubound;
mod val;
