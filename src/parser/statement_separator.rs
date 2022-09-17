use crate::common::QError;
use crate::parser::base::parsers::{and, filter_token, filter_token_by_kind_opt, many_opt, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::specific::{dummy_token, item_p, opt_whitespace_p, whitespace_p, TokenType};

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
        let opt_buf = whitespace_p().parse(reader)?;
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
                Ok(Some(dummy_token(tokenizer)))
            }
        }
    }
}

// ':' <ws>*
fn colon_separator_p() -> impl Parser {
    and(
        filter_token_by_kind_opt(TokenType::Colon),
        opt_whitespace_p(false),
    )
}

// <eol> < ws | eol >*
// TODO rename to _opt
fn eol_separator_p() -> impl Parser {
    and(
        filter_token_by_kind_opt(TokenType::Eol),
        many_opt(eol_or_whitespace_opt()),
    )
}

fn eol_or_whitespace_opt() -> impl Parser {
    filter_token(|token| {
        Ok(token.kind == TokenType::Eol as i32 || token.kind == TokenType::Whitespace as i32)
    })
}
