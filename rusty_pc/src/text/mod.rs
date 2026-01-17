use std::marker::PhantomData;

use crate::{Parser, ParserErrorTrait};

/// An input source that can provide characters.
pub trait CharInput: Clone {
    /// Returns the next character.
    /// Does not advance the position.
    fn char(&self) -> char;

    /// Increase the current position by the given amount.
    fn inc_position_by(self, amount: usize) -> Self;

    /// Is the input at the end of file.
    fn is_eof(&self) -> bool;

    /// Increase the current position by one character.
    fn inc_position(self) -> Self {
        self.inc_position_by(1)
    }
}

/// Parses any char.
/// Fails only on EOF, returning the default parse error (soft error).
pub fn any_char<I, E>() -> AnyChar<I, E>
where
    I: CharInput,
    E: ParserErrorTrait,
{
    AnyChar(PhantomData)
}

/// Parses any char.
/// Fails only on EOF, returning the default parse error (soft error).
pub struct AnyChar<I, E>(PhantomData<(I, E)>);

impl<I, E> Parser<I> for AnyChar<I, E>
where
    I: CharInput,
    E: ParserErrorTrait,
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
