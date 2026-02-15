use std::marker::PhantomData;

use crate::and::Combiner;
use crate::{InputTrait, Parser, ParserErrorTrait};

/// A binary parser that sets the context of the right-side parser
/// based on the value returned by the left-side parser.
///
/// # Regarding `init_context`
///
/// As the right-side context is always set before parsing
/// the right-side, there is no need to use `init_context`.
pub struct ThenWithContextParser<L, R, A, O> {
    left: L,
    right: R,
    combiner: A,
    _marker: PhantomData<O>,
}

impl<L, R, A, O> ThenWithContextParser<L, R, A, O> {
    pub(crate) fn new(left: L, right: R, combiner: A) -> Self {
        Self {
            left,
            right,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<I, C, L, R, A, O> Parser<I, C> for ThenWithContextParser<L, R, A, O>
where
    I: InputTrait,
    L: Parser<I, C>,
    R: Parser<I, L::Output, Error = L::Error>,
    A: Combiner<L::Output, R::Output, O>,
{
    type Output = O;
    type Error = L::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let left = self.left.parse(input)?;
        self.right.set_context(&left);
        match self.right.parse(input) {
            Ok(right) => Ok(self.combiner.combine(left, right)),
            // right-side error is always fatal
            Err(err) => Err(err.to_fatal()),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.left.set_context(ctx);
    }
}
