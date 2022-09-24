use tokenizers::Tokenizer;
use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::undo_pc::Undo;

pub struct SurroundedBy<L, M, R>(L, M, R);

impl<L, M, R> SurroundedBy<L, M, R> {
    pub fn new(left: L, middle: M, right: R) -> Self {
        Self(left, middle, right)
    }
}

impl<L, M, R> HasOutput for SurroundedBy<L, M, R>
where
    M: HasOutput,
{
    type Output = M::Output;
}

impl<L, M, R> Parser for SurroundedBy<L, M, R>
where
    L: Parser,
    M: Parser,
    R: Parser,
    L::Output: Undo,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_first = self.0.parse(tokenizer)?;
        match self.1.parse(tokenizer)? {
            Some(value) => {
                self.2.parse(tokenizer)?;
                Ok(Some(value))
            }
            None => {
                opt_first.undo(tokenizer);
                Ok(None)
            }
        }
    }
}
