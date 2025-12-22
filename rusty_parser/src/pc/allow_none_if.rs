use crate::pc::{ParseResult, Parser};
use crate::{parser_declaration, ParseError};

pub trait AllowNoneIf<I>: Parser<I> {
    fn allow_none_if(self, condition: bool) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized,
    {
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
