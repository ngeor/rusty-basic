use crate::parser::comment::CommentAsString;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use rusty_common::*;

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
            if TokenType::Whitespace.matches(&token) {
                if !found_eol {
                    tokens.push(token);
                }
            } else if TokenType::Eol.matches(&token) {
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
            if TokenType::Whitespace.matches(&token) {
                // skip whitespace
            } else if TokenType::SingleQuote.matches(&token) {
                tokenizer.unread(token);
                return Ok(());
            } else if TokenType::Colon.matches(&token) {
                if sep == TokenType::Unknown {
                    // same line separator
                    sep = TokenType::Colon;
                } else {
                    tokenizer.unread(token);
                    break;
                }
            } else if TokenType::Eol.matches(&token) {
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
    // .allow_none() to accept EOF
    any_token()
        .allow_none()
        .filter(|opt_token| match opt_token {
            Some(token) => {
                TokenType::Colon.matches(token)
                    || TokenType::SingleQuote.matches(token)
                    || TokenType::Eol.matches(token)
            }
            None => true,
        })
        .peek()
}

// TODO review all parsers that return a collection, implement some `accumulate` method
/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl Parser<Output = Vec<Locatable<String>>> + NonOptParser {
    OptAndPC::new(
        whitespace(),
        OptZip::new(Separator::Comment, CommentAsString.with_pos())
            .one_or_more()
            .map(ZipValue::collect_right)
            .allow_default(),
    )
    .keep_right()
}
