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
