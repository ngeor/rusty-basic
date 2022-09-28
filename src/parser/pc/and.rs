use crate::common::QError;
use crate::parser::pc::{HasOutput, Parser, Tokenizer, Undo};

//
// And (with undo if the left parser supports it)
//

// Looks identical to `NonOptSeq2` but that one has already an implementation
// of Parser

pub struct AndPC<A, B>(A, B);

impl<A, B> AndPC<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self(left, right)
    }
}

impl<A, B> HasOutput for AndPC<A, B>
where
    A: HasOutput,
    B: HasOutput,
{
    type Output = (A::Output, B::Output);
}

impl<A, B> Parser for AndPC<A, B>
where
    A: Parser,
    A::Output: Undo,
    B: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => match self.1.parse(tokenizer)? {
                Some(right) => Ok(Some((left, right))),
                None => {
                    left.undo(tokenizer);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}
