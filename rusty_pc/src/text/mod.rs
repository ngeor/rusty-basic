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
