use crate::pc::{ParseResult, Parser};
use crate::ParseError;

pub trait ToOption<I>: Parser<I> {
    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized;
}

impl<I, P> ToOption<I> for P
where
    P: Parser<I>,
{
    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized,
    {
        ToOptionParser::new(self)
    }
}

struct ToOptionParser<P> {
    parser: P,
}

impl<P> ToOptionParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, P> Parser<I> for ToOptionParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err((false, tokenizer, _)) => Ok((tokenizer, None)),
            Err(err) => Err(err),
        }
    }
}
