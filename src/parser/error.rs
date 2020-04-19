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
}

pub fn unexpected<T, S: AsRef<str>>(msg: S, lexeme: LexemeNode) -> Result<T, ParserError> {
    Err(ParserError::Unexpected(msg.as_ref().to_string(), lexeme))
}
