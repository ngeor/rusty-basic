use rusty_pc::{MapErrParser, OrFail, Parser};

use crate::error::ParseError;

pub trait SpecificTrait<I>: Parser<I, Error = ParseError>
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> MapErrParser<Self, ParseError> {
        self.or_fail(ParseError::syntax_error(msg))
    }
}

impl<I, P> SpecificTrait<I> for P where P: Parser<I, Error = ParseError> {}
