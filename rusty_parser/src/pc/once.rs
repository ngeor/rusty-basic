use crate::pc::{ParserOnce, Tokenizer};
use crate::ParseError;

/// A parser that returns the given value only once.
pub fn once_p<V>(value: V) -> Once<V> {
    Once(value)
}

pub struct Once<V>(V);

impl<V> ParserOnce for Once<V> {
    type Output = V;

    fn parse(self, _: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        Ok(self.0)
    }
}
