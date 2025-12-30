use rusty_pc::Parser;

use crate::input::RcStringView;
use crate::specific::built_ins::get::parse_get_or_put;
use crate::specific::*;
use crate::{BuiltInSub, ParseError};

pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
