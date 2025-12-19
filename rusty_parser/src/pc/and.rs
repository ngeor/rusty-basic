use crate::pc::{ParseResult, Parser};
use crate::ParseError;

//
// And (with undo)
//

pub trait And<I: Clone>: Parser<I> {
    /// Parses both the left and the right side.
    /// If the right side fails, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O;

    fn and_tuple<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, R::Output)>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and(right, |l, r| (l, r))
    }

    fn and_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and(right, |l, _| l)
    }

    fn and_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and(right, |_, r| r)
    }
}

impl<I, L> And<I> for L
where
    I: Clone,
    L: Parser<I>,
{
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        AndParser::new(self, right, combiner)
    }
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
