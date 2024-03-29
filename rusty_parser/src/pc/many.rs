//
// Many
//

use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct OneOrMoreParser);

impl<P> Parser for OneOrMoreParser<P>
where
    P: Parser,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let mut result: Vec<P::Output> = Vec::new();
        while let Some(value) = self.parser.parse_opt(tokenizer)? {
            result.push(value);
        }
        if result.is_empty() {
            Err(ParseError::Incomplete)
        } else {
            Ok(result)
        }
    }
}
