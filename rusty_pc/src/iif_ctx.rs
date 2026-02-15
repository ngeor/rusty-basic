use crate::{InputTrait, Parser};

/// Based on the boolean context, parses using the left or the right parser.
pub struct IifCtxParser<L, R> {
    left: L,
    right: R,
    context: bool,
}

impl<L, R> IifCtxParser<L, R> {
    pub fn new<I>(left: L, right: R) -> Self
    where
        I: InputTrait,
        L: Parser<I>,
        R: Parser<I, Output = L::Output, Error = L::Error>,
    {
        Self {
            left,
            right,
            context: false,
        }
    }
}

impl<L, R, I> Parser<I, bool> for IifCtxParser<L, R>
where
    I: InputTrait,
    L: Parser<I>,
    R: Parser<I, Output = L::Output, Error = L::Error>,
{
    type Output = L::Output;
    type Error = L::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if self.context {
            self.left.parse(input)
        } else {
            self.right.parse(input)
        }
    }

    fn set_context(&mut self, ctx: &bool) {
        self.context = *ctx;
    }
}
