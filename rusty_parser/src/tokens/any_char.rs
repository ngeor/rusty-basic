use rusty_pc::{InputTrait, Parser};

use crate::ParserError;
use crate::input::StringView;

/// Parses any char.
/// It never fails. On EOF, it returns the special value `\0`,
/// without incrementing the input's position any further.
pub(super) struct AnyCharOrEof;

impl Parser<StringView> for AnyCharOrEof {
    type Output = char;
    type Error = ParserError;

    fn parse(&mut self, input: &mut StringView) -> Result<Self::Output, ParserError> {
        if input.is_eof() {
            Ok(char::MIN)
        } else {
            Ok(input.read())
        }
    }
}
