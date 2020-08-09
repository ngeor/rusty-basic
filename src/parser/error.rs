use crate::common::*;
use crate::lexer::{Lexeme, LexemeNode, LexerError, LexerErrorNode};

// TODO clarify when we use Unexpected | Unterminated or SyntaxError
// TODO go over all sub-parsers and make sure they honor those semantics and don't backtrack if they aren't supposed do
// TODO add tests for more user friendly errors e.g. "ELSE without IF"
#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An error occurred in the lexer.
    LexerError(LexerError),

    /// Unexpected token. This is a recoverable error.
    Unexpected(String, Lexeme),

    Unterminated(Lexeme),

    SyntaxError(String),
}

pub type ParserErrorNode = ErrorEnvelope<ParserError>;

impl ParserError {
    pub fn unexpected<S: AsRef<str>>(msg: S, lexeme: Lexeme) -> Self {
        ParserError::Unexpected(msg.as_ref().to_string(), lexeme)
    }
}

impl From<LexerError> for ParserError {
    fn from(lexer_error: LexerError) -> Self {
        ParserError::LexerError(lexer_error)
    }
}

pub fn unexpected<T, S: AsRef<str>>(msg: S, lexeme_node: LexemeNode) -> Result<T, ParserErrorNode> {
    let Locatable {
        element: lexeme,
        pos,
    } = lexeme_node;
    Err(ParserError::unexpected(msg, lexeme)).with_err_at(pos)
}

impl From<LexerErrorNode> for ParserErrorNode {
    fn from(l: LexerErrorNode) -> Self {
        l.map(|x| x.into())
    }
}
