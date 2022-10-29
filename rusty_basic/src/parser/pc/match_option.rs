use crate::parser::pc::{ParserOnce, Tokenizer};
use rusty_common::*;

pub fn match_option_p<T, LF, RF>(
    value: Option<T>,
    left_factory: LF,
    right_factory: RF,
) -> MatchOption<T, LF, RF> {
    MatchOption::new(value, left_factory, right_factory)
}

pub struct MatchOption<T, LF, RF> {
    value: Option<T>,
    left_factory: LF,
    right_factory: RF,
}

impl<T, LF, RF> MatchOption<T, LF, RF> {
    pub fn new(value: Option<T>, left_factory: LF, right_factory: RF) -> Self {
        Self {
            value,
            left_factory,
            right_factory,
        }
    }
}

impl<T, LF, RF, L, R> ParserOnce for MatchOption<T, LF, RF>
where
    LF: FnOnce(T) -> L,
    L: ParserOnce,
    RF: FnOnce() -> R,
    R: ParserOnce<Output = L::Output>,
{
    type Output = L::Output;

    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.value {
            Some(value) => {
                let parser = (self.left_factory)(value);
                parser.parse(tokenizer)
            }
            None => {
                let parser = (self.right_factory)();
                parser.parse(tokenizer)
            }
        }
    }
}
