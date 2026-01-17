use std::marker::PhantomData;

use crate::Parser;

pub trait CharInput: Clone {
    fn char(&self) -> char;

    fn inc_position_by(self, amount: usize) -> Self;

    fn is_eof(&self) -> bool;

    fn inc_position(self) -> Self {
        self.inc_position_by(1)
    }
}

/// Parses any char.
/// Fails only on EOF, returning the default parse error (non fatal).
pub struct AnyChar<I, E>(PhantomData<(I, E)>);

pub fn any_char<I, E>() -> AnyChar<I, E>
where
    I: CharInput,
    E: Default,
{
    AnyChar(PhantomData)
}

impl<I, E> Parser<I> for AnyChar<I, E>
where
    I: CharInput,
    E: Default,
{
    type Output = char;
    type Error = E;

    fn parse(&mut self, input: I) -> crate::ParseResult<I, Self::Output, Self::Error> {
        if input.is_eof() {
            crate::default_parse_error(input)
        } else {
            let ch = input.char();
            Ok((input.inc_position(), ch))
        }
    }
}
