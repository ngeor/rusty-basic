use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::base::undo_pc::Undo;
use std::marker::PhantomData;

pub struct SurroundedBy<L, M, R, L1>(L, M, R, L1);

impl<L, M, R, L1> SurroundedBy<L, M, R, L1>
where
    L: HasOutput<Output = L1>,
{
    pub fn new(left: L, middle: M, right: R) -> Self {
        Self(left, middle, right, PhantomData)
    }
}

impl<L, M, R, L1> HasOutput for SurroundedBy<L, M, R, L1>
where
    M: HasOutput,
{
    type Output = M::Output;
}

impl<L, M, R, L1> Parser for SurroundedBy<L, M, R, L1>
where
    L: Parser<Output = L1>,
    M: Parser,
    R: Parser,
    L1: Undo,
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
