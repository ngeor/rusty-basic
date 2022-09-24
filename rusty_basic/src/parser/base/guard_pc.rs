use tokenizers::Tokenizer;
use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};

pub struct GuardPC<L, R>(L, R);

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

impl<L, R> NonOptParser for GuardPC<L, R>
where
    L: NonOptParser,
    R: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer)?;
        self.1.parse_non_opt(tokenizer)
    }
}

pub trait GuardTrait<P> {
    fn then_use(self, parser: P) -> GuardPC<Self, P>
    where
        Self: Sized;
}

impl<S, P> GuardTrait<P> for S {
    fn then_use(self, parser: P) -> GuardPC<Self, P> {
        GuardPC(self, parser)
    }
}
