use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct FilterParser<predicate: F>);

impl<I: Tokenizer + 'static, P, F> Parser<I> for FilterParser<P, F>
where
    P: Parser<I>,
    F: Fn(&P::Output) -> bool,
    P::Output: Undo,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).flat_map(|result| {
            if (self.predicate)(&result) {
                ParseResult::Ok(result)
            } else {
                result.undo(tokenizer);
                ParseResult::Err(ParseError::Incomplete)
            }
        })
    }
}

parser_declaration!(pub struct FilterMapParser<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FilterMapParser<P, F>
where
    P: Parser<I>,
    P::Output: Undo,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .flat_map(|result| match (self.mapper)(&result) {
                Some(value) => ParseResult::Ok(value),
                None => {
                    result.undo(tokenizer);
                    ParseResult::Err(ParseError::Incomplete)
                }
            })
    }
}
