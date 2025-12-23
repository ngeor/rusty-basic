use crate::error::ParseError;
use crate::pc::{ParseResult, ParseResultTrait, Parser, ToOption};

//
// And (without undo)
//

pub trait AndWithoutUndo<I>: Parser<I>
where
    Self: Sized,
{
    /// Parses both the left and the right side.
    /// Be careful: if the right side fails, parsing of the left side
    /// is not undone. This should not be used unless it's certain
    /// that the right side can't fail.
    fn and_without_undo<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        AndWithoutUndoParser::new(self, right, combiner)
    }

    /// Parses the left side and returns the right side.
    /// If the left does not succeed, the right is not parsed.
    /// Be careful: If the right does not succeed, the left is not undone.
    /// This should not be used unless it's certain that the right can't fail.
    /// TODO use a NonOptParser here for the right side.
    fn and_without_undo_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        R: Parser<I>,
    {
        self.and_without_undo(right, |_, right| right)
    }

    /// Parses the left side and optionally the right side.
    /// The combiner function maps the left and (optional) right output to the final result.
    fn and_opt<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        R: Parser<I>,
        F: Fn(Self::Output, Option<R::Output>) -> O,
    {
        self.and_without_undo(right.to_option(), combiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is a tuple of both sides.
    fn and_opt_tuple<R>(
        self,
        right: R,
    ) -> impl Parser<I, Output = (Self::Output, Option<R::Output>)>
    where
        R: Parser<I>,
    {
        self.and_opt(right, |l, r| (l, r))
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the left side's output.
    fn and_opt_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        R: Parser<I>,
    {
        self.and_opt(right, |l, _| l)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the right side's output.
    fn and_opt_keep_right<R>(self, right: R) -> impl Parser<I, Output = Option<R::Output>>
    where
        R: Parser<I>,
    {
        self.and_opt(right, |_, r| r)
    }
}

impl<I, L> AndWithoutUndo<I> for L where L: Parser<I> {}

struct AndWithoutUndoParser<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> AndWithoutUndoParser<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I, L, R, F, O> Parser<I> for AndWithoutUndoParser<L, R, F>
where
    L: Parser<I>,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        self.left.parse(tokenizer).flat_map(|tokenizer, left| {
            self.right
                .parse(tokenizer)
                .map_ok(|right| (self.combiner)(left, right))
        })
    }
}
