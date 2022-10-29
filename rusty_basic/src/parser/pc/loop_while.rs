use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;
use rusty_common::*;

parser_declaration!(pub struct LoopWhile<predicate: F>);

impl<P, F> Parser for LoopWhile<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            match self.parser.parse_opt(tokenizer)? {
                Some(item) => {
                    keep_going = (self.predicate)(&item);
                    // push to the list regardless
                    result.push(item);
                }
                None => {
                    keep_going = false;
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
