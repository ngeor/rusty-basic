use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

/// Parses any char.
/// Fails only on EOF, returning the default parse error (soft error).
pub fn any_char<I, E>() -> AnyChar<I, E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    AnyChar(PhantomData)
}

/// Parses any char.
/// Fails only on EOF, returning the default parse error (soft error).
pub struct AnyChar<I, E>(PhantomData<(I, E)>);

impl<I, E> Parser<I> for AnyChar<I, E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    type Output = char;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if input.is_eof() {
            crate::default_parse_error()
        } else {
            Ok(input.read())
        }
    }
}

/// Parses any char, without reading it.
/// Fails only on EOF, returning the default parse error (soft error).
pub fn peek_char<I, E>() -> PeekChar<I, E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    PeekChar(PhantomData)
}

/// Parses any char, without reading it.
/// Fails only on EOF, returning the default parse error (soft error).
pub struct PeekChar<I, E>(PhantomData<(I, E)>);

impl<I, E> Parser<I> for PeekChar<I, E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    type Output = char;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if input.is_eof() {
            crate::default_parse_error()
        } else {
            Ok(input.peek())
        }
    }
}

/// Parses one specific character.
pub fn one_char<I, E>(ch: char) -> impl Parser<I, Output = char, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    any_char().filter(move |c: &char| *c == ch)
}
