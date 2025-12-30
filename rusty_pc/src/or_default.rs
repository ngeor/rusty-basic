use crate::{ParseResult, Parser};

pub(crate) struct OrDefaultParser<P> {
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
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(tokenizer) {
            Ok(x) => Ok(x),
            Err((false, tokenizer, _)) => Ok((tokenizer, P::Output::default())),
            Err(err) => Err(err),
        }
    }
}
