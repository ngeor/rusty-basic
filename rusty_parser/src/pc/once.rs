use crate::pc::{ParserOnce, Tokenizer};
use crate::ParseError;

/// A parser that returns the given value only once.
pub fn once_p<V>(value: V) -> Once<V> {
    Once(value)
}

pub struct Once<V>(V);

impl<I: Tokenizer + 'static, V> ParserOnce<I> for Once<V> {
    type Output = V;

    fn parse(self, _: &mut I) -> Result<Self::Output, ParseError> {
        Ok(self.0)
    }
}
