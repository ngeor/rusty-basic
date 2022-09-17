use crate::common::QError;
use crate::parser::base::parsers::{
    AndOptTrait, AndTrait, ManyTrait, OrTrait, Parser, TokenPredicate,
};
use crate::parser::base::tokenizers::{Position, Token, Tokenizer};
use crate::parser::specific::{item_p, whitespace, TokenKindParser, TokenType};

// TODO split into two classes one for comments and one for non comments
pub struct StatementSeparator {
    comment_mode: bool,
}

impl StatementSeparator {
    pub fn new(comment_mode: bool) -> Self {
        Self { comment_mode }
    }

    fn parse_comment(
        &self,
        reader: &mut impl Tokenizer,
        mut buf: String,
    ) -> Result<Option<String>, QError> {
        let opt_item = eol_separator_p().parse(reader)?;
        let item = opt_item.unwrap();
        buf.push_str(item.as_str());
        Ok(Some(buf))
    }

    // <ws>* '\'' (undoing it)
    // <ws>* ':' <ws*>
    // <ws>* EOL <ws | eol>*
    fn parse_non_comment(
        &self,
        reader: &mut impl Tokenizer,
        mut buf: String,
    ) -> Result<Option<String>, QError> {
        let opt_item = comment_separator_p()
            .or(colon_separator_p())
            .or(eol_separator_p())
            .parse(reader)?;
        match opt_item {
            Some(item) => {
                buf.push_str(item.as_str());
                Ok(Some(buf))
            }
            _ => Err(QError::syntax_error("Expected: end-of-statement")),
        }
    }
}

impl Parser for StatementSeparator {
    type Output = String;

    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        // skip any whitespace, so that the error will hit the first offending character
        let opt_buf = whitespace().parse(reader)?;
        let buf = opt_buf.unwrap_or_default();
        if self.comment_mode {
            self.parse_comment(reader, buf)
        } else {
            self.parse_non_comment(reader, buf)
        }
    }
}

// '\'' (undoing it)
fn comment_separator_p<R>() -> impl Parser<Output = String> {
    // not adding the ' character in the resulting string because it was already undone
    item_p('\'').peek_reader_item().map(|_| String::new())
}

/// A parser that succeeds on EOF, EOL, colon and comment.
/// Does not undo anything.
pub struct EofOrStatementSeparator;

impl EofOrStatementSeparator {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for EofOrStatementSeparator {
    type Output = Token;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if token.kind == TokenType::Colon as i32
                    || token.kind == TokenType::SingleQuote as i32
                    || token.kind == TokenType::Eol as i32
                {
                    Ok(Some(token))
                } else {
                    tokenizer.unread(token);
                    Ok(None)
                }
            }
            _ => {
                // EOF is accepted
                // TODO fix this so it doesn't need a dummy token
                Ok(Some(dummy_token(tokenizer)))
            }
        }
    }
}

fn dummy_token(tokenizer: &impl Tokenizer) -> Token {
    Token {
        kind: TokenType::Whitespace as i32,
        text: String::new(),
        position: Position {
            begin: tokenizer.position(),
            end: tokenizer.position(),
        },
    }
}

// ':' <ws>*
fn colon_separator_p() -> impl Parser {
    TokenKindParser::new(TokenType::Colon).and_opt(whitespace())
}

// <eol> < ws | eol >*
// TODO rename to _opt
fn eol_separator_p() -> impl Parser {
    TokenKindParser::new(TokenType::Eol).and(EolOrWhitespace.zero_or_more())
}

struct EolOrWhitespace;

impl TokenPredicate for EolOrWhitespace {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Eol as i32 || token.kind == TokenType::Whitespace as i32
    }
}
