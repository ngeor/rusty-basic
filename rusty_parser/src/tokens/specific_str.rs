use rusty_pc::{ParseResult, Parser};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::to_specific_parser::ToSpecificParser;

/// Wraps an [str] so that it can be used as a specific parser.
pub struct SpecificStr<'a> {
    needle: &'a str,
}

impl<'a> ToSpecificParser for &'a str {
    type Parser = SpecificStr<'a>;

    fn to_specific_parser(self) -> Self::Parser {
        SpecificStr { needle: self }
    }
}

impl<'a> Parser<RcStringView> for SpecificStr<'a> {
    type Output = String;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, String, ParseError> {
        parse_specific_str(self.needle, input)
    }
}

/// Wraps a [String] so that it can be used as a specific parser.
pub struct SpecificString {
    needle: String,
}

impl ToSpecificParser for String {
    type Parser = SpecificString;

    fn to_specific_parser(self) -> Self::Parser {
        SpecificString { needle: self }
    }
}

impl Parser<RcStringView> for SpecificString {
    type Output = String;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, String, ParseError> {
        parse_specific_str(&self.needle, input)
    }
}

fn parse_specific_str(
    needle: &str,
    input: RcStringView,
) -> ParseResult<RcStringView, String, ParseError> {
    let mut buffer = String::new();
    let mut success = true;

    // how many characters have we read successfully so far
    let mut read: usize = 0;

    {
        // the input's index when we start reading
        let offset = input.index();

        for expected_ch in needle.chars() {
            // ensure we're not at eof
            if offset + read < input.len() {
                // check the actual character at the input without advancing the input's position
                let actual_ch = input.char_at(offset + read);
                if actual_ch == expected_ch {
                    // it's a match, add it and increment the number of successfully read characters
                    buffer.push(expected_ch);
                    read += 1;
                } else {
                    success = false;
                    break;
                }
            } else {
                success = false;
                break;
            }
        }
    }

    if success {
        Ok((input.inc_position_by(read), buffer))
    } else {
        Err((
            false,
            input,
            ParseError::SyntaxError(format!("Expected: {}", needle)),
        ))
    }
}
