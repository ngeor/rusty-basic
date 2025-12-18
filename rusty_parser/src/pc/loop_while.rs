use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct LoopWhile<predicate: F>);

impl<I: Tokenizer + 'static, P, F> Parser<I> for LoopWhile<P, F>
where
    P: Parser<I>,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            match self.parser.parse(tokenizer) {
                ParseResult::Ok(item) => {
                    keep_going = (self.predicate)(&item);
                    // push to the list regardless
                    result.push(item);
                }
                ParseResult::None | ParseResult::Expected(_) => {
                    keep_going = false;
                }
                ParseResult::Err(err) => return ParseResult::Err(err),
            }
        }
        if result.is_empty() {
            ParseResult::None
        } else {
            ParseResult::Ok(result)
        }
    }
}
