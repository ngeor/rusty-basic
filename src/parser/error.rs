use crate::common::Location;
use crate::lexer::{LexemeNode, LexerError};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An error occurred in the lexer.
    LexerError(LexerError),

    /// An internal error (IO error, unexpected num parsing, etc)
    Internal(String, Location),

    /// Unexpected token
    Unexpected(String, LexemeNode),

    /// A specific token was not found.
    NotFound(String, LexemeNode),
}

impl ParserError {
    pub fn not_found_to_none<T>(self) -> Result<Option<T>, ParserError> {
        match self {
            Self::NotFound(_, _) => Ok(None),
            _ => Err(self),
        }
    }
}
