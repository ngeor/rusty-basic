use crate::common::QError;
use crate::parser::pc::{ParserOnce, Tokenizer};

/// A parser that returns the given value only once.
pub fn once_p<V>(value: V) -> Once<V> {
    Once(value)
}

pub struct Once<V>(V);

impl<V> ParserOnce for Once<V> {
    type Output = V;

    fn parse(self, _: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        Ok(self.0)
    }
}
