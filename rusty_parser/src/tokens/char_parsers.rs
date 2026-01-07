use rusty_pc::{Filter, ParseResult, Parser, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;

pub fn filter<F>(predicate: F) -> impl Parser<RcStringView, Output = char, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    AnyChar.filter(predicate)
}

/// Parses the specific character.
/// If it fails, it returns a non-fatal syntax error
/// with a message like "Expected: z" (where 'z' is the character).
///
/// The [Parser] trait is implemented directly on the [char] type,
/// so this function is just a helper to indicate we're returning it as a [Parser].
pub fn specific(needle: char) -> impl Parser<RcStringView, Output = char, Error = ParseError> {
    needle
}

impl Parser<RcStringView> for char {
    type Output = char;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, ParseError> {
        if input.is_eof() {
            default_parse_error(input)
        } else {
            let ch = input.char();
            if ch == *self {
                Ok((input.inc_position(), ch))
            } else {
                Err((
                    false,
                    input,
                    ParseError::SyntaxError(format!("Expected: {}", self)),
                ))
            }
        }
    }
}

/// Parses any char.
/// Fails only on EOF, returning the default parse error (non fatal).
pub struct AnyChar;

impl Parser<RcStringView> for AnyChar {
    type Output = char;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, ParseError> {
        if input.is_eof() {
            default_parse_error(input)
        } else {
            let ch = input.char();
            Ok((input.inc_position(), ch))
        }
    }
}
