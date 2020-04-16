use crate::common::Location;

#[derive(Debug, PartialEq)]
pub enum LexerError {
    /// An internal error (IO error, unexpected num parsing, etc)
    Internal(String, Location),

    /// An unsupported character.
    /// This is a sign of something missing it the lexer implementation.
    UnsupportedCharacter(char, Location),
}
