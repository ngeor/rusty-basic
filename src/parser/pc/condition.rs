use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};

pub fn condition(flag: bool) -> Condition {
    Condition(flag)
}

pub struct Condition(bool);

impl Parser for Condition {
    type Output = ();

    fn parse(&self, _tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        if self.0 {
            Ok(())
        } else {
            Err(QError::Incomplete)
        }
    }
}
