use crate::pc::*;
use crate::ParseError;

pub struct OrParser<I, O> {
    parsers: Vec<Box<dyn Parser<I, Output = O>>>,
}

impl<I: Tokenizer + 'static, O> OrParser<I, O> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, Output = O>>>) -> Self {
        Self { parsers }
    }
}

impl<I: Tokenizer + 'static, O> Parser<I> for OrParser<I, O> {
    type Output = O;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<O, ParseError> {
        for parser in &self.parsers {
            let result = parser.parse(tokenizer);
            let is_incomplete_err = match &result {
                ParseResult::None | ParseResult::Expected(_) => true,
                _ => false,
            };

            if is_incomplete_err {
                continue;
            } else {
                // return the first Ok result or Fatal error
                return result;
            }
        }

        ParseResult::None
    }
}
