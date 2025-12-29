use crate::input::RcStringView;
use crate::pc::Parser;
use crate::specific::built_ins::get::parse_get_or_put;
use crate::specific::*;
use crate::BuiltInSub;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
