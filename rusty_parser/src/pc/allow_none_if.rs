use crate::error::ParseError;
use crate::parser_declaration;
use crate::pc::{ParseResult, Parser};

pub trait AllowNoneIf<I>: Parser<I>
where
    Self: Sized,
{
    fn allow_none_if(self, condition: bool) -> impl Parser<I, Output = Option<Self::Output>> {
        AllowNoneIfParser::new(self, condition)
    }
}

impl<I, P> AllowNoneIf<I> for P where P: Parser<I> {}

parser_declaration!(
    struct AllowNoneIfParser {
        condition: bool,
    }
);

impl<I, P> Parser<I> for AllowNoneIfParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err((fatal, i, err)) => {
                if self.condition && !fatal {
                    Ok((i, None))
                } else {
                    Err((fatal, i, err))
                }
            }
        }
    }
}
