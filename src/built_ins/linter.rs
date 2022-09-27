use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::QErrorNode;
use crate::linter::NameContext;
use crate::parser::ExpressionNodes;

pub fn lint_sub_call(
    built_in_sub: &BuiltInSub,
    args: &ExpressionNodes,
    name_context: NameContext,
) -> Result<(), QErrorNode> {
    match built_in_sub {
        BuiltInSub::Beep => crate::built_ins::beep::linter::lint(args),
        BuiltInSub::CallAbsolute => Ok(()),
        BuiltInSub::Close => crate::built_ins::close::linter::lint(args),
        BuiltInSub::Cls => crate::built_ins::cls::linter::lint(args),
        BuiltInSub::Color => crate::built_ins::color::linter::lint(args),
        BuiltInSub::Data => crate::built_ins::data::linter::lint(args, name_context),
        BuiltInSub::DefSeg => crate::built_ins::def_seg::linter::lint(args),
        BuiltInSub::Environ => crate::built_ins::environ_sub::linter::lint(args),
        BuiltInSub::Field => crate::built_ins::field::linter::lint(args),
        BuiltInSub::Get => crate::built_ins::get::linter::lint(args),
        BuiltInSub::Input => crate::built_ins::input::linter::lint(args),
        BuiltInSub::Kill => crate::built_ins::kill::linter::lint(args),
        BuiltInSub::LineInput => crate::built_ins::line_input::linter::lint(args),
        BuiltInSub::Locate => crate::built_ins::locate::linter::lint(args),
        BuiltInSub::LSet => crate::built_ins::lset::linter::lint(args),
        BuiltInSub::Name => crate::built_ins::name::linter::lint(args),
        BuiltInSub::Open => crate::built_ins::open::linter::lint(args),
        BuiltInSub::Poke => crate::built_ins::poke::linter::lint(args),
        BuiltInSub::Put => crate::built_ins::put::linter::lint(args),
        BuiltInSub::Read => crate::built_ins::read::linter::lint(args),
        BuiltInSub::Screen => Ok(()),
        BuiltInSub::ViewPrint => crate::built_ins::view_print::linter::lint(args),
        BuiltInSub::Width => crate::built_ins::width::linter::lint(args),
    }
}

pub fn lint_function_call(
    built_in: &BuiltInFunction,
    args: &ExpressionNodes,
) -> Result<(), QErrorNode> {
    match built_in {
        BuiltInFunction::Chr => crate::built_ins::chr::linter::lint(args),
        BuiltInFunction::Cvd => crate::built_ins::cvd::linter::lint(args),
        BuiltInFunction::Environ => crate::built_ins::environ_fn::linter::lint(args),
        BuiltInFunction::Eof => crate::built_ins::eof::linter::lint(args),
        BuiltInFunction::Err => crate::built_ins::err::linter::lint(args),
        BuiltInFunction::InKey => crate::built_ins::inkey::linter::lint(args),
        BuiltInFunction::InStr => crate::built_ins::instr::linter::lint(args),
        BuiltInFunction::LBound => crate::built_ins::lbound::linter::lint(args),
        BuiltInFunction::LCase => crate::built_ins::lcase::linter::lint(args),
        BuiltInFunction::Left => crate::built_ins::left::linter::lint(args),
        BuiltInFunction::Len => crate::built_ins::len::linter::lint(args),
        BuiltInFunction::LTrim => crate::built_ins::ltrim::linter::lint(args),
        BuiltInFunction::Mid => crate::built_ins::mid_fn::linter::lint(args),
        BuiltInFunction::Mkd => crate::built_ins::mkd::linter::lint(args),
        BuiltInFunction::Peek => crate::built_ins::peek::linter::lint(args),
        BuiltInFunction::Right => crate::built_ins::right::linter::lint(args),
        BuiltInFunction::RTrim => crate::built_ins::rtrim::linter::lint(args),
        BuiltInFunction::Space => crate::built_ins::space::linter::lint(args),
        BuiltInFunction::Str => crate::built_ins::str_fn::linter::lint(args),
        BuiltInFunction::String_ => crate::built_ins::string_fn::linter::lint(args),
        BuiltInFunction::UBound => crate::built_ins::ubound::linter::lint(args),
        BuiltInFunction::UCase => crate::built_ins::ucase::linter::lint(args),
        BuiltInFunction::Val => crate::built_ins::val::linter::lint(args),
        BuiltInFunction::VarPtr => crate::built_ins::varptr::linter::lint(args),
        BuiltInFunction::VarSeg => crate::built_ins::varseg::linter::lint(args),
    }
}
