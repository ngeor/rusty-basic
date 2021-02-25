use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{FileAccess, FileHandle, FileMode, QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::stdlib::Stdlib;
use crate::parser::{ElementType, TypeQualifier, UserDefinedType, UserDefinedTypes};
use crate::variant::{Variant, MAX_INTEGER, MAX_LONG};
use std::convert::TryInto;

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

#[derive(Default)]
pub struct RunSubResult {
    pub halt: bool,
}

impl RunSubResult {
    pub fn new_halt() -> Self {
        Self { halt: true }
    }
}

pub fn run_sub<S: InterpreterTrait>(
    s: &BuiltInSub,
    interpreter: &mut S,
) -> Result<RunSubResult, QErrorNode> {
    match s {
        BuiltInSub::End | BuiltInSub::System => Ok(RunSubResult::new_halt()),
        _ => run_not_terminating_sub(s, interpreter).map(|_| RunSubResult::default()),
    }
}

fn run_not_terminating_sub<S: InterpreterTrait>(
    s: &BuiltInSub,
    interpreter: &mut S,
) -> Result<(), QErrorNode> {
    match s {
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Input => input::run(interpreter).with_err_no_pos(),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter).with_err_no_pos(),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
        BuiltInSub::End | BuiltInSub::System => {
            panic!("Should not have been called")
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
