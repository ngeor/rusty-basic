use crate::pc::{Parser, Tokenizer, Undo};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct FilterParser<predicate: F>);

impl<P, F> Parser for FilterParser<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
    P::Output: Undo,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let result = self.parser.parse(tokenizer)?;
        if (self.predicate)(&result) {
            Ok(result)
        } else {
            result.undo(tokenizer);
            Err(ParseError::Incomplete)
        }
    }
}

parser_declaration!(pub struct FilterMapParser<mapper: F>);

impl<P, F, U> Parser for FilterMapParser<P, F>
where
    P: Parser,
    P::Output: Undo,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let result = self.parser.parse(tokenizer)?;
        match (self.mapper)(&result) {
            Some(value) => Ok(value),
            None => {
                result.undo(tokenizer);
                Err(ParseError::Incomplete)
            }
        }
    }
}
