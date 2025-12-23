use crate::error::ParseError;
use crate::pc::{ParseResult, Parser, ToOption};

//
// And (with undo)
//

pub trait And<I>: Parser<I>
where
    Self: Sized,
    I: Clone,
{
    /// Parses both the left and the right side.
    /// If the right side fails, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        AndParser::new(self, right, combiner)
    }

    fn and_tuple<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, R::Output)>
    where
        R: Parser<I>,
    {
        self.and(right, |l, r| (l, r))
    }

    fn and_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        R: Parser<I>,
    {
        self.and(right, |l, _| l)
    }

    fn and_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        R: Parser<I>,
    {
        self.and(right, |_, r| r)
    }

    fn surround<L, R>(self, left: L, right: R) -> impl Parser<I, Output = Self::Output>
    where
        L: Parser<I>,
        R: Parser<I>,
    {
        left.and_keep_right(self).and_keep_left(right)
    }
}

impl<I, L> And<I> for L
where
    I: Clone,
    L: Parser<I>,
{
}

struct AndParser<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> AndParser<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I, L, R, F, O> Parser<I> for AndParser<L, R, F>
where
    I: Clone,
    L: Parser<I>,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.left.parse(tokenizer.clone()) {
            Ok((input, left)) => {
                match self.right.parse(input) {
                    Ok((input, right)) => Ok((input, (self.combiner)(left, right))),
                    // return original input here
                    Err((false, _, err)) => Err((false, tokenizer, err)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The combiner function is used to create the final result.
pub fn opt_and<I, L, R, F, O>(
    left: impl Parser<I, Output = L>,
    right: impl Parser<I, Output = R>,
    combiner: F,
) -> impl Parser<I, Output = O>
where
    I: Clone,
    F: Fn(Option<L>, R) -> O,
{
    left.to_option().and(right, combiner)
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The result is a tuple of the (optional) left side and the right side.
pub fn opt_and_tuple<I, L, R>(
    left: impl Parser<I, Output = L>,
    right: impl Parser<I, Output = R>,
) -> impl Parser<I, Output = (Option<L>, R)>
where
    I: Clone,
{
    opt_and(left, right, |l, r| (l, r))
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The result is the right side only, the left is discarded.
pub fn opt_and_keep_right<I, L, R>(
    left: impl Parser<I, Output = L>,
    right: impl Parser<I, Output = R>,
) -> impl Parser<I, Output = R>
where
    I: Clone,
{
    opt_and(left, right, |_, r| r)
}
