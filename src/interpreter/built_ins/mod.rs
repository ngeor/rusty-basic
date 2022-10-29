mod beep;
mod chr;
mod close;
mod cls;
mod color;
mod cvd;
mod data;
mod def_seg;
mod environ_fn;
mod environ_sub;
mod eof;
mod err;
mod field;
mod get;
mod inkey;
mod input;
mod instr;
mod kill;
mod lbound;
mod lcase;
mod left;
mod len;
mod line_input;
mod locate;
mod lset;
mod ltrim;
mod mid_fn;
mod mkd;
mod name;
mod open;
mod peek;
mod poke;
mod put;
mod read;
mod right;
mod rtrim;
mod space;
mod str_fn;
mod string_fn;
mod ubound;
mod ucase;
mod val;
mod varptr;
mod varseg;
mod view_print;
mod width;

use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::{BuiltInFunction, BuiltInSub};

pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QError> {
    match s {
        BuiltInSub::Beep => beep::run(interpreter),
        BuiltInSub::CallAbsolute => Ok(()),
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Cls => cls::run(interpreter),
        BuiltInSub::Color => color::run(interpreter),
        BuiltInSub::Data => data::run(interpreter),
        BuiltInSub::DefSeg => def_seg::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Field => field::run(interpreter),
        BuiltInSub::Get => get::run(interpreter),
        BuiltInSub::Input => input::run(interpreter),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter),
        BuiltInSub::Locate => locate::run(interpreter),
        BuiltInSub::LSet => lset::run(interpreter),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
        BuiltInSub::Poke => poke::run(interpreter),
        BuiltInSub::Put => put::run(interpreter),
        BuiltInSub::Read => read::run(interpreter),
        BuiltInSub::Screen => Ok(()),
        BuiltInSub::ViewPrint => view_print::run(interpreter),
        BuiltInSub::Width => width::run(interpreter),
    }
}

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QError> {
    match f {
        BuiltInFunction::Chr => chr::run(interpreter),
        BuiltInFunction::Cvd => cvd::run(interpreter),
        BuiltInFunction::Environ => environ_fn::run(interpreter),
        BuiltInFunction::Eof => eof::run(interpreter),
        BuiltInFunction::Err => err::run(interpreter),
        BuiltInFunction::InKey => inkey::run(interpreter),
        BuiltInFunction::InStr => instr::run(interpreter),
        BuiltInFunction::LBound => lbound::run(interpreter),
        BuiltInFunction::LCase => lcase::run(interpreter),
        BuiltInFunction::Left => left::run(interpreter),
        BuiltInFunction::Len => len::run(interpreter),
        BuiltInFunction::LTrim => ltrim::run(interpreter),
        BuiltInFunction::Mid => mid_fn::run(interpreter),
        BuiltInFunction::Mkd => mkd::run(interpreter),
        BuiltInFunction::Peek => peek::run(interpreter),
        BuiltInFunction::Right => right::run(interpreter),
        BuiltInFunction::RTrim => rtrim::run(interpreter),
        BuiltInFunction::Space => space::run(interpreter),
        BuiltInFunction::Str => str_fn::run(interpreter),
        BuiltInFunction::String_ => string_fn::run(interpreter),
        BuiltInFunction::UBound => ubound::run(interpreter),
        BuiltInFunction::UCase => ucase::run(interpreter),
        BuiltInFunction::Val => val::run(interpreter),
        BuiltInFunction::VarPtr => varptr::run(interpreter),
        BuiltInFunction::VarSeg => varseg::run(interpreter),
    }
}
