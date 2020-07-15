use crate::common::{HasLocation, Location};
use crate::lexer::{LexemeNode, LexerError};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An error occurred in the lexer.
    LexerError(LexerError),

    /// An internal error (IO error, unexpected num parsing, etc)
    Internal(String, Location),

    /// Unexpected token
    Unexpected(String, LexemeNode),
}

pub fn unexpected<T, S: AsRef<str>>(msg: S, lexeme: LexemeNode) -> Result<T, ParserError> {
    Err(ParserError::Unexpected(msg.as_ref().to_string(), lexeme))
}

impl HasLocation for ParserError {
    fn location(&self) -> Location {
        match self {
            Self::LexerError(l) => l.location(),
            Self::Internal(_, pos) => pos.clone(),
            Self::Unexpected(_, l) => l.location(),
        }
    }
}
