use crate::{ParseResult, Parser, parser1};

parser1!(
    trait ToOption;
    struct ToOptionParser;
    fn to_option
);

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
