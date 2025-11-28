use crate::built_ins::get::parse_get_or_put;
use crate::pc::{Parser, Tokenizer};
use crate::{BuiltInSub, Keyword, Statement};

pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    parse_get_or_put(Keyword::Put, BuiltInSub::Put)
}
