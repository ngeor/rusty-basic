use rusty_pc::text::CharInput;
use rusty_pc::{ParseResult, Parser};

use crate::ParserError;
use crate::input::RcStringView;

/// Parses any char.
/// It never fails. On EOF, it returns the special value `\0`,
/// without incrementing the input's position any further.
pub(super) struct AnyCharOrEof;

impl Parser<RcStringView> for AnyCharOrEof {
    type Output = char;
    type Error = ParserError;

    fn parse(
        &mut self,
        input: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParserError> {
        if input.is_eof() {
            Ok((input, char::MIN))
        } else {
            let ch = input.char();
            Ok((input.inc_position(), ch))
        }
    }
}
