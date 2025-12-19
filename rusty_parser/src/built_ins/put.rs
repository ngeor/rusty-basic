use crate::built_ins::get::parse_get_or_put;
use crate::pc::{Parser, RcStringView};
use crate::{BuiltInSub, Keyword, Statement};

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
