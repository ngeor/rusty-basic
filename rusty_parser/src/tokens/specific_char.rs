use rusty_pc::{ParseResult, Parser, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::to_specific_parser::ToSpecificParser;

/// Parses the specific character.
/// If it fails, it returns a non-fatal syntax error
/// with a message like "Expected: z" (where 'z' is the character).
pub struct SpecificChar {
    needle: char,
}

impl ToSpecificParser for char {
    type Parser = SpecificChar;

    fn to_specific_parser(self) -> Self::Parser {
        SpecificChar { needle: self }
    }
}

impl Parser<RcStringView> for SpecificChar {
    type Output = char;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, ParseError> {
        if input.is_eof() {
            default_parse_error(input)
        } else {
            let ch = input.char();
            if ch == self.needle {
                Ok((input.inc_position(), ch))
            } else {
                Err((
                    false,
                    input,
                    ParseError::SyntaxError(format!("Expected: {}", self.needle)),
                ))
            }
        }
    }
}
