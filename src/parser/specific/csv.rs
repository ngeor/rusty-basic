use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::specific::{item_p, surrounded_by_opt_ws};

pub fn csv_one_or_more<P>(parser: P) -> impl Parser<Output = Vec<P::Output>>
where
    P: Parser,
{
    DelimitedPC {
        parser,
        delimiter: comma_surrounded_by_opt_ws(),
    }
}

pub fn comma_surrounded_by_opt_ws() -> impl Parser<Output = Token> {
    surrounded_by_opt_ws(item_p(','))
}

struct DelimitedPC<A, B> {
    parser: A,
    delimiter: B,
}

impl<A, B> HasOutput for DelimitedPC<A, B>
where
    A: HasOutput,
{
    type Output = Vec<A::Output>;
}

impl<A, B> Parser for DelimitedPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(first) => self.after_element(tokenizer, vec![first]).map(Some),
            None => Ok(None),
        }
    }
}

impl<A, B> DelimitedPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn after_element(
        &self,
        tokenizer: &mut impl Tokenizer,
        collected: Vec<A::Output>,
    ) -> Result<Vec<A::Output>, QError> {
        match self.delimiter.parse(tokenizer)? {
            Some(_) => self.after_delimiter(tokenizer, collected),
            None => Ok(collected),
        }
    }

    fn after_delimiter(
        &self,
        tokenizer: &mut impl Tokenizer,
        mut collected: Vec<A::Output>,
    ) -> Result<Vec<A::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(next) => {
                collected.push(next);
                self.after_element(tokenizer, collected)
            }
            None => Err(QError::syntax_error("Trailing comma")),
        }
    }
}
