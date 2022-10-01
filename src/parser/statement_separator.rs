use crate::common::*;
use crate::parser::comment::CommentAsString;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

pub enum Separator {
    /// `<ws>* EOL <ws | eol>*`
    Comment,

    /// ````text
    /// <ws>* '\'' (undoing it)
    /// <ws>* ':' <ws*>
    /// <ws>* EOL <ws | eol>*
    /// ```
    NonComment,
}

impl Parser for Separator {
    type Output = ();
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self {
            Self::Comment => CommentSeparator.parse(tokenizer),
            Self::NonComment => CommonSeparator.parse(tokenizer),
        }
    }
}

struct CommentSeparator;

impl Parser for CommentSeparator {
    type Output = ();
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut tokens: TokenList = vec![];
        let mut found_eol = false;
        while let Some(token) = tokenizer.read()? {
            if token.kind == TokenType::Whitespace as i32 {
                if !found_eol {
                    tokens.push(token);
                }
            } else if token.kind == TokenType::Eol as i32 {
                found_eol = true;
                tokens.clear();
            } else {
                tokenizer.unread(token);
                break;
            }
        }
        if found_eol {
            Ok(())
        } else {
            tokens.undo(tokenizer);
            Err(QError::Incomplete)
        }
    }
}

struct CommonSeparator;

impl Parser for CommonSeparator {
    type Output = ();
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut sep = TokenType::Unknown;
        while let Some(token) = tokenizer.read()? {
            if token.kind == TokenType::Whitespace as i32 {
                // skip whitespace
            } else if token.kind == TokenType::SingleQuote as i32 {
                tokenizer.unread(token);
                return Ok(());
            } else if token.kind == TokenType::Colon as i32 {
                if sep == TokenType::Unknown {
                    // same line separator
                    sep = TokenType::Colon;
                } else {
                    tokenizer.unread(token);
                    break;
                }
            } else if token.kind == TokenType::Eol as i32 {
                if sep == TokenType::Unknown || sep == TokenType::Eol {
                    // multiline separator
                    sep = TokenType::Eol;
                } else {
                    tokenizer.unread(token);
                    break;
                }
            } else {
                tokenizer.unread(token);
                break;
            }
        }
        if sep != TokenType::Unknown {
            Ok(())
        } else {
            Err(QError::Incomplete)
        }
    }
}

pub fn peek_eof_or_statement_separator() -> impl Parser<Output = ()> {
    PeekStatementSeparatorOrEof(StatementSeparator2)
}

struct PeekStatementSeparatorOrEof<P>(P);

impl<P> Parser for PeekStatementSeparatorOrEof<P>
where
    P: TokenPredicate,
{
    type Output = ();
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<(), QError> {
        match tokenizer.read()? {
            Some(token) => {
                let found_it = self.0.test(&token);
                tokenizer.unread(token);
                if found_it {
                    Ok(())
                } else {
                    Err(QError::Incomplete)
                }
            }
            None => Ok(()),
        }
    }
}

struct StatementSeparator2;

impl TokenPredicate for StatementSeparator2 {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Colon as i32
            || token.kind == TokenType::SingleQuote as i32
            || token.kind == TokenType::Eol as i32
    }
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl Parser<Output = Vec<Locatable<String>>> {
    CommentsAndWhitespace.preceded_by_opt_ws()
}

struct CommentsAndWhitespace;

impl Parser for CommentsAndWhitespace {
    type Output = Vec<Locatable<String>>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<Locatable<String>> = vec![];
        let sep = Separator::Comment;
        let parser = CommentAsString.with_pos();
        let mut found_separator = true;
        let mut found_comment = true;
        while found_separator || found_comment {
            found_separator = sep.parse_opt(tokenizer)?.is_some();
            match parser.parse_opt(tokenizer)? {
                Some(comment) => {
                    result.push(comment);
                    found_comment = true;
                }
                _ => {
                    found_comment = false;
                }
            }
        }
        Ok(result)
    }
}
