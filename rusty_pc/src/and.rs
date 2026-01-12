use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait, SetContext, ToOption, ToOptionParser};

//
// And (with undo)
//

pub trait And<I: InputTrait, C>: Parser<I, C>
where
    Self: Sized,
{
    /// Parses both the left and the right side.
    /// If the right side fails with a soft error, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> AndParser<Self, R, F, O>
    where
        R: Parser<I, C, Error = Self::Error>,
        F: Combiner<Self::Output, R::Output, O>,
    {
        AndParser::new(self, right, combiner)
    }

    fn and_tuple<R>(self, right: R) -> AndParser<Self, R, TupleCombiner, (Self::Output, R::Output)>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, TupleCombiner)
    }

    fn and_keep_left<R>(self, right: R) -> AndParser<Self, R, KeepLeftCombiner, Self::Output>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, KeepLeftCombiner)
    }

    fn and_keep_right<R>(self, right: R) -> AndParser<Self, R, KeepRightCombiner, R::Output>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, KeepRightCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The combiner function maps the left and (optional) right output to the final result.
    fn and_opt<R, F, O>(self, right: R, combiner: F) -> AndParser<Self, ToOptionParser<R>, F, O>
    where
        R: Parser<I, C, Error = Self::Error>,
        F: Combiner<Self::Output, Option<R::Output>, O>,
    {
        self.and(right.to_option(), combiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is a tuple of both sides.
    fn and_opt_tuple<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, TupleCombiner, (Self::Output, Option<R::Output>)>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, TupleCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the left side's output.
    fn and_opt_keep_left<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, KeepLeftCombiner, Self::Output>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, KeepLeftCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the right side's output.
    fn and_opt_keep_right<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, KeepRightCombiner, Option<R::Output>>
    where
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, KeepRightCombiner)
    }
}

impl<I, C, L> And<I, C> for L
where
    I: InputTrait,
    L: Parser<I, C>,
{
}

pub struct AndParser<L, R, F, O> {
    left: L,
    right: R,
    combiner: F,
    _marker: PhantomData<O>,
}

impl<L, R, F, O> AndParser<L, R, F, O> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<I, C, L, R, F, O> Parser<I, C> for AndParser<L, R, F, O>
where
    I: InputTrait,
    L: Parser<I, C>,
    R: Parser<I, C, Error = L::Error>,
    F: Combiner<L::Output, R::Output, O>,
{
    type Output = O;
    type Error = L::Error;

    fn parse(&mut self, tokenizer: &mut I) -> Result<Self::Output, Self::Error> {
        let original_position = tokenizer.get_position();
        let left = self.left.parse(tokenizer)?;
        match self.right.parse(tokenizer) {
            Ok(right) => Ok(self.combiner.combine(left, right)),
            Err(err) => {
                if !err.is_fatal() {
                    tokenizer.set_position(original_position);
                }
                Err(err)
            }
        }
    }
}

impl<C, L, R, F, O> SetContext<C> for AndParser<L, R, F, O>
where
    L: SetContext<C>,
    R: SetContext<C>,
    C: Clone,
{
    fn set_context(&mut self, context: C) {
        self.left.set_context(context.clone());
        self.right.set_context(context);
    }
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The combiner function is used to create the final result.
pub fn opt_and<I, L, R, E, F, O>(
    left: impl Parser<I, Output = L, Error = E>,
    right: impl Parser<I, Output = R, Error = E>,
    combiner: F,
) -> impl Parser<I, Output = O, Error = E>
where
    I: InputTrait,
    E: ParserErrorTrait,
    F: Combiner<Option<L>, R, O>,
{
    left.to_option().and(right, combiner)
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The result is a tuple of the (optional) left side and the right side.
pub fn opt_and_tuple<I, L, R, E>(
    left: impl Parser<I, Output = L, Error = E>,
    right: impl Parser<I, Output = R, Error = E>,
) -> impl Parser<I, Output = (Option<L>, R), Error = E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    opt_and(left, right, TupleCombiner)
}

/// Parses the left side optionally and then the right side.
/// If the right side fails, the left side is reverted too.
/// The result is the right side only, the left is discarded.
pub fn opt_and_keep_right<I, L, R, E>(
    left: impl Parser<I, Output = L, Error = E>,
    right: impl Parser<I, Output = R, Error = E>,
) -> impl Parser<I, Output = R, Error = E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    opt_and(left, right, KeepRightCombiner)
}

/// Combines two values into one.
pub trait Combiner<L, R, O> {
    /// Combines two values into one.
    fn combine(&self, left: L, right: R) -> O;
}

/// Combines two values into a tuple.
pub struct TupleCombiner;

impl<L, R> Combiner<L, R, (L, R)> for TupleCombiner {
    fn combine(&self, left: L, right: R) -> (L, R) {
        (left, right)
    }
}

/// Combines two values by keeping the left value.
pub struct KeepLeftCombiner;

impl<L, R> Combiner<L, R, L> for KeepLeftCombiner {
    fn combine(&self, left: L, _right: R) -> L {
        left
    }
}

/// Combines two values by keeping the right value.
pub struct KeepRightCombiner;

impl<L, R> Combiner<L, R, R> for KeepRightCombiner {
    fn combine(&self, _left: L, right: R) -> R {
        right
    }
}

/// Ignores both left and right value and returns `()`.
pub struct IgnoringBothCombiner;

impl<L, R> Combiner<L, R, ()> for IgnoringBothCombiner {
    fn combine(&self, _left: L, _right: R) {}
}

// Combiner implementation for `Fn`.

impl<L, R, O, F> Combiner<L, R, O> for F
where
    F: Fn(L, R) -> O,
{
    fn combine(&self, left: L, right: R) -> O {
        (self)(left, right)
    }
}

/// Combines two vectors by concatenating them into one.
pub struct VecCombiner;

impl<L> Combiner<Vec<L>, Vec<L>, Vec<L>> for VecCombiner {
    fn combine(&self, mut left: Vec<L>, mut right: Vec<L>) -> Vec<L> {
        left.append(&mut right);
        left
    }
}

/// Combines two strings into one by concatenating them.
/// Supports also an Optional String with a String.
pub struct StringCombiner;

impl Combiner<String, String, String> for StringCombiner {
    fn combine(&self, mut left: String, right: String) -> String {
        left.push_str(&right);
        left
    }
}

impl Combiner<Option<String>, String, String> for StringCombiner {
    fn combine(&self, left: Option<String>, right: String) -> String {
        match left {
            Some(left) => self.combine(left, right),
            None => right,
        }
    }
}

impl Combiner<char, Vec<char>, String> for StringCombiner {
    fn combine(&self, left: char, right: Vec<char>) -> String {
        let mut result = String::from(left);
        for ch in right {
            result.push(ch);
        }
        result
    }
}
