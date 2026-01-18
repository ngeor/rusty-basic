use std::marker::PhantomData;

use crate::{Combiner, InputTrait, Parser, ParserErrorTrait, SetContext};

/// A binary parser that sets the context of the right-side parser
/// based on the value returned by the left-side parser.
///
/// # Regarding `init_context`
///
/// As the right-side context is always set before parsing
/// the right-side, there is no need to use `init_context`.
pub trait ThenWithContext<I: InputTrait, C>: Parser<I, C> {
    /// Combines this parser with another parser,
    /// setting the other parser's context before parsing
    /// based on the result of this parser.
    ///
    /// The right-side parser is treated as a 'complete' parser,
    /// i.e. soft errors will be converted to fatal.
    ///
    /// # Arguments
    ///
    /// * self: this parser (the left-side parser)
    /// * other: the right-side parser
    /// * ctx_projection: a function that maps the left-side result into the right-side context
    fn then_with_in_context<R, F, A, O, CR>(
        self,
        other: R,
        ctx_projection: F,
        combiner: A,
    ) -> ThenWithContextParser<Self, R, F, A, O>
    where
        Self: Sized,
        R: Parser<I, CR, Error = Self::Error>,
        F: Fn(&Self::Output) -> CR,
        A: Combiner<Self::Output, R::Output, O>,
    {
        ThenWithContextParser::new(self, other, ctx_projection, combiner)
    }
}

impl<I, C, P> ThenWithContext<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
{
}

pub struct ThenWithContextParser<L, R, F, A, O> {
    left: L,
    right: R,
    ctx_projection: F,
    combiner: A,
    _marker: PhantomData<O>,
}

impl<L, R, F, A, O> ThenWithContextParser<L, R, F, A, O> {
    pub fn new(left: L, right: R, ctx_projection: F, combiner: A) -> Self {
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
