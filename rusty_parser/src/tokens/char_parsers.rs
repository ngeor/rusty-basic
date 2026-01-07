use rusty_pc::{Filter, ParseResult, Parser, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;

pub fn any() -> impl Parser<RcStringView, Output = char, Error = ParseError> {
    AnyChar
}

pub fn filter<F>(predicate: F) -> impl Parser<RcStringView, Output = char, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    any().filter(predicate)
}

pub fn specific(needle: char) -> impl Parser<RcStringView, Output = char, Error = ParseError> {
    filter(move |ch| *ch == needle)
}

struct AnyChar;

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
