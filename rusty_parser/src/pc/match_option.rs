use crate::pc::{ParserOnce, Tokenizer};
use crate::ParseError;

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

impl<I: Tokenizer + 'static, T, LF, RF, L, R> ParserOnce<I> for MatchOption<T, LF, RF>
where
    LF: FnOnce(T) -> L,
    L: ParserOnce<I>,
    RF: FnOnce() -> R,
    R: ParserOnce<I, Output = L::Output>,
{
    type Output = L::Output;

    fn parse(self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
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
