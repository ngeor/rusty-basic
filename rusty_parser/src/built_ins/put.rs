use rusty_pc::Parser;

use crate::built_ins::get::parse_get_or_put;
use crate::input::RcStringView;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParserError> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
