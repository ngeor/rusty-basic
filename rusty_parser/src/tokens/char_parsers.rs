use rusty_pc::{Filter, ParseResult, Parser, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;

pub fn filter_or_err<F>(
    predicate: F,
    err: ParseError,
) -> impl Parser<RcStringView, Output = char, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    AnyChar.filter_or_err(predicate, err)
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
