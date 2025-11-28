use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct LoopWhile<predicate: F>);

impl<I: Tokenizer + 'static, P, F> Parser<I> for LoopWhile<P, F>
where
    P: Parser<I>,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
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
            Err(ParseError::Incomplete)
        } else {
            Ok(result)
        }
    }
}
