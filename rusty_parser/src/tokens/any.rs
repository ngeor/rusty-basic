use rusty_pc::{ParseResult, Parser, default_parse_error};

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::tokens::peek_token;

/// Returns Ok(()) if we're at EOF,
/// otherwise an incomplete result,
/// without modifying the input.
pub fn detect_eof() -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    EofDetector
}

struct EofDetector;

impl Parser<RcStringView> for EofDetector {
    type Output = ();
    type Error = ParseError;

    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        match peek_token().parse(tokenizer) {
            Ok((i, _)) => default_parse_error(i),
            Err((false, i, _)) => Ok((i, ())),
            Err(err) => Err(err),
        }
    }
}
