use crate::common::{HasLocation, Location};
use crate::lexer::{LexemeNode, LexerError};

// TODO clarify when we use Unexpected | Unterminated or SyntaxError
// TODO go over all sub-parsers and make sure they honor those semantics and don't backtrack if they aren't supposed do
// TODO add tests for more user friendly errors e.g. "ELSE without IF"
#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An error occurred in the lexer.
    LexerError(LexerError),

    /// An internal error (IO error, unexpected num parsing, etc)
    Internal(String, Location),

    /// Unexpected token
    Unexpected(String, LexemeNode),

    Unterminated(LexemeNode),

    SyntaxError(LexemeNode),
}

pub fn unexpected<T, S: AsRef<str>>(msg: S, lexeme: LexemeNode) -> Result<T, ParserError> {
    Err(ParserError::Unexpected(msg.as_ref().to_string(), lexeme))
}

impl HasLocation for ParserError {
    fn location(&self) -> Location {
        match self {
            Self::LexerError(l) => l.location(),
            Self::Internal(_, pos) => pos.clone(),
            Self::Unexpected(_, l) | Self::Unterminated(l) | Self::SyntaxError(l) => l.location(),
        }
    }
}

impl From<LexerError> for ParserError {
    fn from(lexer_error: LexerError) -> Self {
        Self::LexerError(lexer_error)
    }
}
