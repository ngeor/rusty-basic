//
// Many
//

use crate::common::QError;
use crate::parser::pc::{NonOptParser, Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct OneOrMoreParser);

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

parser_declaration!(struct ZeroOrMoreParser);

impl<P> Parser for ZeroOrMoreParser<P>
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
        Ok(result)
    }
}

impl<P> NonOptParser for ZeroOrMoreParser<P> where P: Parser {}
