use crate::parser::built_ins::get::parse_get_or_put;
use crate::parser::pc::Parser;
use crate::parser::{BuiltInSub, Keyword, Statement};

pub fn parse() -> impl Parser<Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
