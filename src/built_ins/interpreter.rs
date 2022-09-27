use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QError> {
    match s {
        BuiltInSub::Beep => crate::built_ins::beep::interpreter::run(interpreter),
        BuiltInSub::CallAbsolute => Ok(()),
        BuiltInSub::Close => crate::built_ins::close::interpreter::run(interpreter),
        BuiltInSub::Cls => crate::built_ins::cls::interpreter::run(interpreter),
        BuiltInSub::Color => crate::built_ins::color::interpreter::run(interpreter),
        BuiltInSub::Data => crate::built_ins::data::interpreter::run(interpreter),
        BuiltInSub::DefSeg => crate::built_ins::def_seg::interpreter::run(interpreter),
        BuiltInSub::Environ => crate::built_ins::environ_sub::interpreter::run(interpreter),
        BuiltInSub::Field => crate::built_ins::field::interpreter::run(interpreter),
        BuiltInSub::Get => crate::built_ins::get::interpreter::run(interpreter),
        BuiltInSub::Input => crate::built_ins::input::interpreter::run(interpreter),
        BuiltInSub::Kill => crate::built_ins::kill::interpreter::run(interpreter),
        BuiltInSub::LineInput => crate::built_ins::line_input::interpreter::run(interpreter),
        BuiltInSub::Locate => crate::built_ins::locate::interpreter::run(interpreter),
        BuiltInSub::LSet => crate::built_ins::lset::interpreter::run(interpreter),
        BuiltInSub::Name => crate::built_ins::name::interpreter::run(interpreter),
        BuiltInSub::Open => crate::built_ins::open::interpreter::run(interpreter),
        BuiltInSub::Poke => crate::built_ins::poke::interpreter::run(interpreter),
        BuiltInSub::Put => crate::built_ins::put::interpreter::run(interpreter),
        BuiltInSub::Read => crate::built_ins::read::interpreter::run(interpreter),
        BuiltInSub::Screen => Ok(()),
        BuiltInSub::ViewPrint => crate::built_ins::view_print::interpreter::run(interpreter),
        BuiltInSub::Width => crate::built_ins::width::interpreter::run(interpreter),
    }
}

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QError> {
    match f {
        BuiltInFunction::Chr => crate::built_ins::chr::interpreter::run(interpreter),
        BuiltInFunction::Cvd => crate::built_ins::cvd::interpreter::run(interpreter),
        BuiltInFunction::Environ => crate::built_ins::environ_fn::interpreter::run(interpreter),
        BuiltInFunction::Eof => crate::built_ins::eof::interpreter::run(interpreter),
        BuiltInFunction::Err => crate::built_ins::err::interpreter::run(interpreter),
        BuiltInFunction::InKey => crate::built_ins::inkey::interpreter::run(interpreter),
        BuiltInFunction::InStr => crate::built_ins::instr::interpreter::run(interpreter),
        BuiltInFunction::LBound => crate::built_ins::lbound::interpreter::run(interpreter),
        BuiltInFunction::LCase => crate::built_ins::lcase::interpreter::run(interpreter),
        BuiltInFunction::Left => crate::built_ins::left::interpreter::run(interpreter),
        BuiltInFunction::Len => crate::built_ins::len::interpreter::run(interpreter),
        BuiltInFunction::LTrim => crate::built_ins::ltrim::interpreter::run(interpreter),
        BuiltInFunction::Mid => crate::built_ins::mid_fn::interpreter::run(interpreter),
        BuiltInFunction::Mkd => crate::built_ins::mkd::interpreter::run(interpreter),
        BuiltInFunction::Peek => crate::built_ins::peek::interpreter::run(interpreter),
        BuiltInFunction::Right => crate::built_ins::right::interpreter::run(interpreter),
        BuiltInFunction::RTrim => crate::built_ins::rtrim::interpreter::run(interpreter),
        BuiltInFunction::Space => crate::built_ins::space::interpreter::run(interpreter),
        BuiltInFunction::Str => crate::built_ins::str_fn::interpreter::run(interpreter),
        BuiltInFunction::String_ => crate::built_ins::string_fn::interpreter::run(interpreter),
        BuiltInFunction::UBound => crate::built_ins::ubound::interpreter::run(interpreter),
        BuiltInFunction::UCase => crate::built_ins::ucase::interpreter::run(interpreter),
        BuiltInFunction::Val => crate::built_ins::val::interpreter::run(interpreter),
        BuiltInFunction::VarPtr => crate::built_ins::varptr::interpreter::run(interpreter),
        BuiltInFunction::VarSeg => crate::built_ins::varseg::interpreter::run(interpreter),
    }
}
