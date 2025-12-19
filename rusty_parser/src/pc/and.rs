use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::ParseError;

//
// And (with undo)
//

pub trait And<I: Tokenizer + 'static>: Parser<I> {
    /// Parses both the left and the right side.
    /// If the right side fails, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O;

    fn and_tuple<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, R::Output)>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |l, r| (l, r))
    }

    fn and_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |l, _| l)
    }

    fn and_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |_, r| r)
    }
}

impl<I, L> And<I> for L
where
    I: Tokenizer + 'static,
    L: Parser<I>,
    L::Output: Undo,
{
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        Self::Output: Undo,
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

impl<I: Tokenizer + 'static, L, R, F, O> Parser<I> for AndParser<L, R, F>
where
    L: Parser<I>,
    L::Output: Undo,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.left.parse(tokenizer) {
            ParseResult::Ok(left) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((self.combiner)(left, right)),
                ParseResult::None => {
                    left.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Expected(s) => {
                    left.undo(tokenizer);
                    ParseResult::Expected(s)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::None => ParseResult::None,
            ParseResult::Expected(s) => ParseResult::Expected(s),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
