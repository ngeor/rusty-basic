use crate::common::QError;
use crate::parser::pc::{HasOutput, NonOptParser, Parser, Tokenizer};

pub struct AndDemandLookingBack<L, F>(L, F);

impl<L, F> AndDemandLookingBack<L, F> {
    pub fn new(left: L, right_factory: F) -> Self {
        Self(left, right_factory)
    }
}

impl<L, F, R> HasOutput for AndDemandLookingBack<L, F>
where
    L: HasOutput,
    R: HasOutput,
    F: Fn(&L::Output) -> R,
{
    type Output = (L::Output, R::Output);
}

impl<L, F, R> Parser for AndDemandLookingBack<L, F>
where
    L: Parser,
    R: NonOptParser,
    F: Fn(&L::Output) -> R,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(first) => {
                let right_parser = (self.1)(&first);
                let second = right_parser.parse_non_opt(tokenizer)?;
                Ok(Some((first, second)))
            }
            None => Ok(None),
        }
    }
}
