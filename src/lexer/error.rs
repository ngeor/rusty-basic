use super::LexemeNode;
use crate::common::Location;

#[derive(Debug, PartialEq)]
pub enum LexerError {
    Internal(String, Location),

    /// An unsupported character.
    /// This is a sign of something missing it the lexer implementation.
    UnsupportedCharacter(char, Location),

    /// Unexpected lexeme.
    Unexpected(String, LexemeNode),
}
