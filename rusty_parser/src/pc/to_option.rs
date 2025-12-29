use crate::pc::{ParseResult, Parser};

pub trait ToOption<I>: Parser<I>
where
    Self: Sized,
{
    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>, Error = Self::Error> {
        ToOptionParser::new(self)
    }
}

impl<I, P> ToOption<I> for P where P: Parser<I> {}

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
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(tokenizer) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err((false, tokenizer, _)) => Ok((tokenizer, None)),
            Err(err) => Err(err),
        }
    }
}
