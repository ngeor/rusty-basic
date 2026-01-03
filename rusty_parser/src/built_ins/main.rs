use rusty_pc::*;

use crate::input::RcStringView;
use crate::{Expression, ParseError, Statement};

// Parses built-in subs which have a special syntax.
pub fn built_in_sub_call_p() -> OrParser<RcStringView, (), Statement, ParseError> {
    OrParser::new(vec![
        Box::new(super::close::parse()),
        Box::new(super::color::parse()),
        Box::new(super::data::parse()),
        Box::new(super::def_seg::parse()),
        Box::new(super::field::parse()),
        Box::new(super::get::parse()),
        Box::new(super::input::parse()),
        Box::new(super::line_input::parse()),
        Box::new(super::locate::parse()),
        Box::new(super::lset::parse()),
        Box::new(super::name::parse()),
        Box::new(super::open::parse()),
        Box::new(super::put::parse()),
        Box::new(super::read::parse()),
        Box::new(super::view_print::parse()),
        Box::new(super::width::parse()),
    ])
}

// needed for built-in functions that are also keywords (e.g. LEN), so they
// cannot be parsed by the `word` module.
pub fn built_in_function_call_p()
-> impl Parser<RcStringView, Output = Expression, Error = ParseError> {
    OrParser::new(vec![
        Box::new(super::len::parse()),
        Box::new(super::string_fn::parse()),
    ])
}
