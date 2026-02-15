use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

/// And (with undo)
pub struct AndParser<L, R, F, O> {
    left: L,
    right: R,
    combiner: F,
    _marker: PhantomData<O>,
}

impl<L, R, F, O> AndParser<L, R, F, O> {
    pub(crate) fn new(left: L, right: R, combiner: F) -> Self {
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
    C: Clone,
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
                if err.is_soft() {
                    tokenizer.set_position(original_position);
                }
                Err(err)
            }
        }
    }

    fn set_context(&mut self, context: C) {
        self.left.set_context(context.clone());
        self.right.set_context(context);
    }
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

/// Combines items into a vector.
pub struct VecCombiner;

/// Combines two items into a vector.
impl<L> Combiner<L, L, Vec<L>> for VecCombiner {
    fn combine(&self, left: L, right: L) -> Vec<L> {
        vec![left, right]
    }
}

/// Combines two vectors by concatenating them into one.
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

impl Combiner<char, char, String> for StringCombiner {
    fn combine(&self, left: char, right: char) -> String {
        let mut result = String::from(left);
        result.push(right);
        result
    }
}

impl Combiner<char, Option<char>, String> for StringCombiner {
    fn combine(&self, left: char, right: Option<char>) -> String {
        match right {
            Some(right) => {
                let mut result = String::from(left);
                result.push(right);
                result
            }
            None => String::from(left),
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
