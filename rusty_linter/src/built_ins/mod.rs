mod arg_validation;
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

use rusty_common::Position;
use rusty_parser::{BuiltInFunction, BuiltInSub, Expressions};

use crate::core::{LintErrorPos, ScopeKind};

pub fn lint_sub_call(
    built_in_sub: &BuiltInSub,
    pos: Position,
    args: &Expressions,
    scope_kind: ScopeKind,
) -> Result<(), LintErrorPos> {
    match built_in_sub {
        BuiltInSub::Beep => beep::lint(args, pos),
        BuiltInSub::CallAbsolute => Ok(()),
        BuiltInSub::Close => close::lint(args),
        BuiltInSub::Cls => cls::lint(args, pos),
        BuiltInSub::Color => color::lint(args, pos),
        BuiltInSub::Data => data::lint(args, scope_kind, pos),
        BuiltInSub::DefSeg => def_seg::lint(args, pos),
        BuiltInSub::Environ => environ_sub::lint(args, pos),
        BuiltInSub::Field => field::lint(args, pos),
        BuiltInSub::Get => get::lint(args, pos),
        BuiltInSub::Input => input::lint(args, pos),
        BuiltInSub::Kill => kill::lint(args, pos),
        BuiltInSub::LineInput => line_input::lint(args, pos),
        BuiltInSub::Locate => locate::lint(args, pos),
        BuiltInSub::LSet => lset::lint(args, pos),
        BuiltInSub::Name => name::lint(args, pos),
        BuiltInSub::Open => open::lint(args, pos),
        BuiltInSub::Poke => poke::lint(args, pos),
        BuiltInSub::Put => put::lint(args, pos),
        BuiltInSub::Read => read::lint(args, pos),
        BuiltInSub::Screen => Ok(()),
        BuiltInSub::ViewPrint => view_print::lint(args, pos),
        BuiltInSub::Width => width::lint(args, pos),
    }
}

pub fn lint_function_call(
    built_in: &BuiltInFunction,
    pos: Position,
    args: &Expressions,
) -> Result<(), LintErrorPos> {
    match built_in {
        BuiltInFunction::Chr => chr::lint(args, pos),
        BuiltInFunction::Cvd => cvd::lint(args, pos),
        BuiltInFunction::Environ => environ_fn::lint(args, pos),
        BuiltInFunction::Eof => eof::lint(args, pos),
        BuiltInFunction::Err => err::lint(args, pos),
        BuiltInFunction::InKey => inkey::lint(args, pos),
        BuiltInFunction::InStr => instr::lint(args, pos),
        BuiltInFunction::LBound => lbound::lint(args, pos),
        BuiltInFunction::LCase => lcase::lint(args, pos),
        BuiltInFunction::Left => left::lint(args, pos),
        BuiltInFunction::Len => len::lint(args, pos),
        BuiltInFunction::LTrim => ltrim::lint(args, pos),
        BuiltInFunction::Mid => mid_fn::lint(args, pos),
        BuiltInFunction::Mkd => mkd::lint(args, pos),
        BuiltInFunction::Peek => peek::lint(args, pos),
        BuiltInFunction::Right => right::lint(args, pos),
        BuiltInFunction::RTrim => rtrim::lint(args, pos),
        BuiltInFunction::Space => space::lint(args, pos),
        BuiltInFunction::Str => str_fn::lint(args, pos),
        BuiltInFunction::String => string_fn::lint(args, pos),
        BuiltInFunction::UBound => ubound::lint(args, pos),
        BuiltInFunction::UCase => ucase::lint(args, pos),
        BuiltInFunction::Val => val::lint(args, pos),
        BuiltInFunction::VarPtr => varptr::lint(args, pos),
        BuiltInFunction::VarSeg => varseg::lint(args, pos),
    }
}
