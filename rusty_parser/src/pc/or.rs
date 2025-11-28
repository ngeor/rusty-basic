use crate::pc::*;
use crate::ParseError;
use crate::ParserErrorTrait;

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
    fn parse(&self, tokenizer: &mut I) -> Result<O, ParseError> {
        for parser in &self.parsers {
            let result = parser.parse(tokenizer);
            let mut is_incomplete_err = false;
            if let Err(e) = &result {
                is_incomplete_err = e.is_incomplete();
            }

            if is_incomplete_err {
                continue;
            } else {
                // return the first Ok result or Fatal error
                return result;
            }
        }

        Err(ParseError::Incomplete)
    }
}

impl<I: Tokenizer + 'static, O> NonOptParser<I> for OrParser<I, O> {}

pub struct OrParserOnce<L, R> {
    left: L,
    right: R,
}

impl<L, R> OrParserOnce<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<I: Tokenizer + 'static, O, L, R> ParserOnce<I> for OrParserOnce<L, R>
where
    L: ParserOnce<I, Output = O>,
    R: ParserOnce<I, Output = O>,
{
    type Output = O;
    fn parse(self, tokenizer: &mut I) -> Result<O, ParseError> {
        let result = self.left.parse(tokenizer);
        let mut is_incomplete_err = false;
        if let Err(e) = &result {
            is_incomplete_err = e.is_incomplete();
        }

        if is_incomplete_err {
            return self.right.parse(tokenizer);
        } else {
            // return the first Ok result or Fatal error
            return result;
        }
    }
}
