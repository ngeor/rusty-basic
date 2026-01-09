use rusty_pc::{ParseResult, Parser, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;

/// Parses any char.
/// Fails only on EOF, returning the default parse error (non fatal).
pub(super) struct AnyChar;

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
