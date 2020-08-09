use crate::common::ErrorEnvelope;

#[derive(Debug, PartialEq)]
pub enum LexerError {
    /// An internal error (IO error, unexpected num parsing, etc)
    Internal(String),

    /// An unsupported character.
    /// This is a sign of something missing it the lexer implementation.
    UnsupportedCharacter(char),
}

pub type LexerErrorNode = ErrorEnvelope<LexerError>;

impl From<std::io::Error> for LexerError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
