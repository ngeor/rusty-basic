use crate::{ParseResult, Parser, parser1};

parser1!(
    trait OrDefault;
    struct OrDefaultParser;
    fn or_default
);

impl<I, C, P> Parser<I, C> for OrDefaultParser<P>
where
    P: Parser<I, C>,
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
