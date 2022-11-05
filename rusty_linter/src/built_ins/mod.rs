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

use crate::NameContext;
use rusty_common::QErrorPos;
use rusty_parser::{BuiltInFunction, BuiltInSub, Expressions};

pub fn lint_sub_call(
    built_in_sub: &BuiltInSub,
    args: &Expressions,
    name_context: NameContext,
) -> Result<(), QErrorPos> {
    match built_in_sub {
        BuiltInSub::Beep => beep::lint(args),
        BuiltInSub::CallAbsolute => Ok(()),
        BuiltInSub::Close => close::lint(args),
        BuiltInSub::Cls => cls::lint(args),
        BuiltInSub::Color => color::lint(args),
        BuiltInSub::Data => data::lint(args, name_context),
        BuiltInSub::DefSeg => def_seg::lint(args),
        BuiltInSub::Environ => environ_sub::lint(args),
        BuiltInSub::Field => field::lint(args),
        BuiltInSub::Get => get::lint(args),
        BuiltInSub::Input => input::lint(args),
        BuiltInSub::Kill => kill::lint(args),
        BuiltInSub::LineInput => line_input::lint(args),
        BuiltInSub::Locate => locate::lint(args),
        BuiltInSub::LSet => lset::lint(args),
        BuiltInSub::Name => name::lint(args),
        BuiltInSub::Open => open::lint(args),
        BuiltInSub::Poke => poke::lint(args),
        BuiltInSub::Put => put::lint(args),
        BuiltInSub::Read => read::lint(args),
        BuiltInSub::Screen => Ok(()),
        BuiltInSub::ViewPrint => view_print::lint(args),
        BuiltInSub::Width => width::lint(args),
    }
}

pub fn lint_function_call(built_in: &BuiltInFunction, args: &Expressions) -> Result<(), QErrorPos> {
    match built_in {
        BuiltInFunction::Chr => chr::lint(args),
        BuiltInFunction::Cvd => cvd::lint(args),
        BuiltInFunction::Environ => environ_fn::lint(args),
        BuiltInFunction::Eof => eof::lint(args),
        BuiltInFunction::Err => err::lint(args),
        BuiltInFunction::InKey => inkey::lint(args),
        BuiltInFunction::InStr => instr::lint(args),
        BuiltInFunction::LBound => lbound::lint(args),
        BuiltInFunction::LCase => lcase::lint(args),
        BuiltInFunction::Left => left::lint(args),
        BuiltInFunction::Len => len::lint(args),
        BuiltInFunction::LTrim => ltrim::lint(args),
        BuiltInFunction::Mid => mid_fn::lint(args),
        BuiltInFunction::Mkd => mkd::lint(args),
        BuiltInFunction::Peek => peek::lint(args),
        BuiltInFunction::Right => right::lint(args),
        BuiltInFunction::RTrim => rtrim::lint(args),
        BuiltInFunction::Space => space::lint(args),
        BuiltInFunction::Str => str_fn::lint(args),
        BuiltInFunction::String => string_fn::lint(args),
        BuiltInFunction::UBound => ubound::lint(args),
        BuiltInFunction::UCase => ucase::lint(args),
        BuiltInFunction::Val => val::lint(args),
        BuiltInFunction::VarPtr => varptr::lint(args),
        BuiltInFunction::VarSeg => varseg::lint(args),
    }
}
