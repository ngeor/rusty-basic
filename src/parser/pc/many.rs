//
// Many
//

use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(pub struct OneOrMoreParser);

impl<P> Parser for OneOrMoreParser<P>
where
    P: Parser,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse_opt(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() {
            Err(QError::Incomplete)
        } else {
            Ok(result)
        }
    }
}