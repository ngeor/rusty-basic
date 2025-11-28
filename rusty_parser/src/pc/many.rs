//
// Many
//

use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct OneOrMoreParser);

impl<I: Tokenizer + 'static, P> Parser<I> for OneOrMoreParser<P>
where
    P: Parser<I>,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
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
