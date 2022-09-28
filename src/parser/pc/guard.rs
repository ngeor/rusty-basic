use crate::common::QError;
use crate::parser::pc::*;

pub struct GuardPC<L, R>(L, R);

impl<L, R> GuardPC<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<L, R> HasOutput for GuardPC<L, R>
where
    R: HasOutput,
{
    type Output = R::Output;
}

impl<L, R> Parser for GuardPC<L, R>
where
    L: Parser,
    R: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(_) => self.1.parse_non_opt(tokenizer).map(Some),
            None => Ok(None),
        }
    }
}
