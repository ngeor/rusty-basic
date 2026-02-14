//! Top-level parsers that read directly from the input source.
use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

/// Reads the next element of the input.
/// Returns the default parse error upon EOF.
pub fn read_p<I, E>() -> impl Parser<I, Output = I::Output, Error = E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    ReadParser(PhantomData)
}

/// Reads the next element of the input.
/// Returns the default parse error upon EOF.
struct ReadParser<E>(PhantomData<E>);

impl<I, E> Parser<I> for ReadParser<E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    type Output = I::Output;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if input.is_eof() {
            crate::default_parse_error()
        } else {
            Ok(input.read())
        }
    }

    fn set_context(&mut self, _ctx: ()) {}
}

/// Peeks the next element of the input.
/// Returns the default parse error upon EOF.
pub fn peek_p<I, E>() -> impl Parser<I, Output = I::Output, Error = E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    PeekParser(PhantomData)
}

/// Peeks the next element of the input.
/// Returns the default parse error upon EOF.
struct PeekParser<E>(PhantomData<E>);

impl<I, E> Parser<I> for PeekParser<E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    type Output = I::Output;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if input.is_eof() {
            crate::default_parse_error()
        } else {
            Ok(input.peek())
        }
    }

    fn set_context(&mut self, _ctx: ()) {
        // do nothing
    }
}

/// Parses one specific element.
pub fn one_p<I, O, E>(needle: O) -> impl Parser<I, Output = O, Error = E>
where
    I: InputTrait<Output = O>,
    O: PartialEq,
    E: ParserErrorTrait,
{
    read_p().filter(move |x: &O| *x == needle)
}

/// Parses one of the given elements.
/// Note that the operation has O(n) complexity.
pub fn one_of_p<I, O, E>(needles: &[O]) -> impl Parser<I, Output = O, Error = E>
where
    I: InputTrait<Output = O>,
    O: PartialEq,
    E: ParserErrorTrait,
{
    read_p().filter(move |x: &O| needles.contains(x))
}

/// Tests if an element is acceptable.
/// Used in order to create a parser that skips while the test succeeds,
/// without accumulating the input. The typical use is whitespace.
pub trait IgnoringPredicate {
    /// The element type.
    type Output;

    /// Tests if the element is acceptable.
    fn test(&self, element: Self::Output) -> bool;

    /// Creates a parser for this predicate.
    fn parser<E>(self, allow_empty: bool) -> IgnoringParser<Self, E>
    where
        Self: Sized,
        E: ParserErrorTrait,
    {
        IgnoringParser::new(self, allow_empty)
    }
}

/// A parser that parses while the predicate succeeds.
pub struct IgnoringParser<P, E> {
    predicate: P,
    allow_empty: bool,
    _marker: PhantomData<E>,
}

impl<P, E> IgnoringParser<P, E> {
    pub fn new(predicate: P, allow_empty: bool) -> Self {
        Self {
            predicate,
            allow_empty,
            _marker: PhantomData,
        }
    }
}

impl<I, C, P, E> Parser<I, C> for IgnoringParser<P, E>
where
    I: InputTrait,
    P: IgnoringPredicate<Output = I::Output>,
    E: ParserErrorTrait,
{
    type Output = ();
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let mut count = 0;
        while !input.is_eof() && self.predicate.test(input.peek()) {
            count += 1;
            input.read();
        }
        if count == 0 && !self.allow_empty {
            crate::default_parse_error()
        } else {
            Ok(())
        }
    }
}
