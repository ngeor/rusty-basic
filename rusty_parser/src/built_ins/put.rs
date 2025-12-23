use crate::built_ins::built_in_sub::BuiltInSub;
use crate::built_ins::get::parse_get_or_put;
use crate::pc::{Parser, RcStringView};
use crate::specific::*;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
