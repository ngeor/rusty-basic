use std::marker::PhantomData;

use crate::and::Combiner;
use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

/// A binary parser that sets the context of the right-side parser
/// based on the value returned by the left-side parser.
///
/// # Regarding `init_context`
///
/// As the right-side context is always set before parsing
/// the right-side, there is no need to use `init_context`.
pub struct ThenWithContextParser<L, R, F, A, O> {
    left: L,
    right: R,
    ctx_projection: F,
    combiner: A,
    _marker: PhantomData<O>,
}

impl<L, R, F, A, O> ThenWithContextParser<L, R, F, A, O> {
    pub(crate) fn new(left: L, right: R, ctx_projection: F, combiner: A) -> Self {
        Self {
            left,
            right,
            ctx_projection,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<I, C, L, R, F, A, O, CR> Parser<I, C> for ThenWithContextParser<L, R, F, A, O>
where
    I: InputTrait,
    L: Parser<I, C>,
    R: Parser<I, CR, Error = L::Error> + SetContext<CR>,
    F: Fn(&L::Output) -> CR,
    A: Combiner<L::Output, R::Output, O>,
{
    type Output = O;
    type Error = L::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let left = self.left.parse(input)?;
        let ctx = (self.ctx_projection)(&left);
        self.right.set_context(ctx);
        match self.right.parse(input) {
            Ok(right) => Ok(self.combiner.combine(left, right)),
            // right-side error is always fatal
            Err(err) => Err(err.to_fatal()),
        }
    }
}
