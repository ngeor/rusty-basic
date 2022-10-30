use crate::built_ins::get::parse_get_or_put;
use crate::pc::Parser;
use crate::{BuiltInSub, Keyword, Statement};

pub fn parse() -> impl Parser<Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
