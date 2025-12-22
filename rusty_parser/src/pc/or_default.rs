use crate::pc::{ParseResult, Parser};
use crate::ParseError;

pub trait OrDefault<I>: Parser<I>
where
    Self: Sized,
    Self::Output: Default,
{
    fn or_default(self) -> impl Parser<I, Output = Self::Output> {
        OrDefaultParser::new(self)
    }
}

impl<I, P> OrDefault<I> for P
where
    P: Parser<I>,
    P::Output: Default,
{
}

struct OrDefaultParser<P> {
    parser: P,
}

impl<P> OrDefaultParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, P> Parser<I> for OrDefaultParser<P>
where
    P: Parser<I>,
    P::Output: Default,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(x) => Ok(x),
            Err((false, tokenizer, _)) => Ok((tokenizer, P::Output::default())),
            Err(err) => Err(err),
        }
    }
}
