use crate::common::QError;
use crate::parser::pc::{HasOutput, NonOptParser, Parser, Tokenizer};

pub struct LoopWhile<P, F>(P, F);

impl<P, F> HasOutput for LoopWhile<P, F>
where
    P: HasOutput,
{
    type Output = Vec<P::Output>;
}

impl<P, F> NonOptParser for LoopWhile<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            match self.0.parse(tokenizer)? {
                Some(item) => {
                    keep_going = (self.1)(&item);
                    // push to the list regardless
                    result.push(item);
                }
                None => {
                    keep_going = false;
                }
            }
        }
        Ok(result)
    }
}

impl<P, F> Parser for LoopWhile<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let result = self.parse_non_opt(tokenizer)?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

pub trait LoopWhileTrait
where
    Self: Sized + HasOutput,
{
    fn loop_while<F>(self, f: F) -> LoopWhile<Self, F>
    where
        F: Fn(&Self::Output) -> bool;
}

impl<P> LoopWhileTrait for P
where
    P: HasOutput,
{
    fn loop_while<F>(self, f: F) -> LoopWhile<Self, F>
    where
        F: Fn(&P::Output) -> bool,
    {
        LoopWhile(self, f)
    }
}

//
// non opt
//

pub struct LoopWhileNonOpt<P, F>(P, F);

impl<P, F> LoopWhileNonOpt<P, F> {
    pub fn new(parser: P, predicate: F) -> Self {
        Self(parser, predicate)
    }
}

impl<P, F> HasOutput for LoopWhileNonOpt<P, F>
where
    P: HasOutput,
{
    type Output = Vec<P::Output>;
}

impl<P, F> NonOptParser for LoopWhileNonOpt<P, F>
where
    P: NonOptParser,
    F: Fn(&P::Output) -> bool,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            let item = self.0.parse_non_opt(tokenizer)?;
            keep_going = (self.1)(&item);
            // push to the list regardless
            result.push(item);
        }
        Ok(result)
    }
}
