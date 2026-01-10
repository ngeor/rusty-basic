use std::cell::RefCell;
use std::marker::PhantomData;

use crate::{Combiner, ParseResult, Parser, SetContext};

/// This parser is a binary parser that sets
/// the context of the left-side parser
/// based on the value returned by the right-side parser.
///
/// Such a scenario is most useful inside loops
/// where the left-side parser needs to be aware
/// of what was parsed by the right-side parser
/// on the previous iteration.
///
/// # Warning
///
/// The context of the left-side parser will probably need to be initialized
/// before the first usage (see `init_context`). Also be mindful of using
/// inside loops, where the context should be reset before the loop starts.
///
/// # Generic parameters
///
/// * L: The left-side parser
/// * R: The right-side parser
/// * X: The right-side parser's context, which can be different than the rest.
/// * F: The function that creates the left-side context out of the right-side result.
/// * A: The combiner that combines the two results into a single value.
/// * O: The output of the parser (the combined result)
pub struct ThenWithLeftParser<L, R, X, F, A, O> {
    left: RefCell<L>,
    right: R,
    right_context_mapper: F,
    combiner: A,
    _marker: PhantomData<(X, O)>,
}

impl<L, R, X, F, A, O> ThenWithLeftParser<L, R, X, F, A, O> {
    pub fn new(left: L, right: R, right_context_mapper: F, combiner: A) -> Self {
        Self {
            left: RefCell::new(left),
            right,
            right_context_mapper,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<I, C, L, R, X, F, A, O> Parser<I, C> for ThenWithLeftParser<L, R, X, F, A, O>
where
    L: Parser<I, C> + SetContext<C>,
    R: Parser<I, X, Error = L::Error>,
    F: Fn(&R::Output) -> C,
    A: Combiner<L::Output, R::Output, O>,
{
    type Output = O;
    type Error = L::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        // the block is done in order to relinquish the borrow
        let (input, left) = { self.left.borrow().parse(input)? };
        match self.right.parse(input) {
            Ok((input, right)) => {
                // create context from right
                let left_context = (self.right_context_mapper)(&right);
                // pass it back to left
                self.left.borrow_mut().set_context(left_context);
                // return the result
                Ok((input, self.combiner.combine(left, right)))
            }
            // right-side error is always fatal
            Err((_, i, err)) => Err((true, i, err)),
        }
    }
}

impl<C, L, R, X, F, A, O> SetContext<C> for ThenWithLeftParser<L, R, X, F, A, O>
where
    L: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        // on purpose not setting the context to the right side,
        // as it is the one that it is supposed to generate the context
        // of the left side.
        self.left.borrow_mut().set_context(ctx);
    }
}
