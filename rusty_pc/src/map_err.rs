use crate::{ParseResult, Parser, parser_declaration};

parser_declaration!(pub(crate)struct NoIncompleteParser);

impl<I, P> Parser<I> for NoIncompleteParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((_, i, err)) => Err((true, i, err)),
        }
    }
}

pub trait Errors<I>: Parser<I>
where
    Self: Sized,
{
    fn or_fail(self, err: Self::Error) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self::Error: Clone,
    {
        OrFailParser::new(self, err)
    }
}

impl<I, P> Errors<I> for P where P: Parser<I> {}

struct OrFailParser<P, E> {
    parser: P,
    err: E,
}

impl<P, E> OrFailParser<P, E> {
    pub fn new(parser: P, err: E) -> Self {
        Self { parser, err }
    }
}

impl<I, P, E> Parser<I> for OrFailParser<P, E>
where
    P: Parser<I, Error = E>,
    E: Clone,
{
    type Output = P::Output;
    type Error = E;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((false, i, _)) => Err((true, i, self.err.clone())),
            Err(err) => Err(err),
        }
    }
}
