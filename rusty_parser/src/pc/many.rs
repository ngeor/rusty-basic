//
// Many
//

use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct OneOrMoreParser);

impl<I: Tokenizer + 'static, P> Parser<I> for OneOrMoreParser<P>
where
    P: Parser<I>,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse_opt(tokenizer) {
                ParseResult::Ok(Some(value)) => {
                    result.push(value);
                }
                ParseResult::Ok(None) | ParseResult::None | ParseResult::Expected(_) => {
                    break;
                }
                ParseResult::Err(err) => {
                    return ParseResult::Err(err);
                }
            }
        }

        if result.is_empty() {
            ParseResult::None
        } else {
            ParseResult::Ok(result)
        }
    }
}
