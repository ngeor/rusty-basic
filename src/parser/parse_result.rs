use crate::lexer::{LexemeNode, LexerError};

pub enum ParseResult<T> {
    Match(T),
    NoMatch(LexemeNode),
}

impl<T> ParseResult<T> {
    pub fn demand<S: AsRef<str>>(self, msg: S) -> Result<T, LexerError> {
        match self {
            ParseResult::Match(x) => Ok(x),
            ParseResult::NoMatch(lexeme) => {
                Err(LexerError::Unexpected(msg.as_ref().to_string(), lexeme))
            }
        }
    }
}

impl<T> From<LexemeNode> for Result<ParseResult<T>, LexerError> {
    fn from(failed_lexeme_node: LexemeNode) -> Result<ParseResult<T>, LexerError> {
        Ok(ParseResult::NoMatch(failed_lexeme_node))
    }
}
