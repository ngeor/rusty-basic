use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser, TokenPredicate};
use crate::parser::base::tokenizers::{Token, TokenList, Tokenizer};
use crate::parser::base::undo_pc::Undo;
use crate::parser::specific::TokenType;

pub enum Separator {
    Comment,
    NonComment,
}

impl HasOutput for Separator {
    type Output = ();
}

impl Parser for Separator {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self {
            Self::Comment => CommentSeparator.parse(tokenizer),
            Self::NonComment => CommonSeparator.parse(tokenizer),
        }
    }
}

// <ws>* EOL <ws | eol>*
struct CommentSeparator;

impl HasOutput for CommentSeparator {
    type Output = ();
}

impl Parser for CommentSeparator {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
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
            Ok(Some(()))
        } else {
            tokens.undo(tokenizer);
            Ok(None)
        }
    }
}

// <ws>* '\'' (undoing it)
// <ws>* ':' <ws*>
// <ws>* EOL <ws | eol>*
struct CommonSeparator;

impl HasOutput for CommonSeparator {
    type Output = ();
}

impl Parser for CommonSeparator {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut sep = TokenType::Unknown;
        while let Some(token) = tokenizer.read()? {
            if token.kind == TokenType::Whitespace as i32 {
                // skip whitespace
            } else if token.kind == TokenType::SingleQuote as i32 {
                tokenizer.unread(token);
                return Ok(Some(()));
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
        Ok(if sep != TokenType::Unknown {
            Some(())
        } else {
            None
        })
    }
}

pub fn peek_eof_or_statement_separator() -> impl Parser<Output = ()> {
    PeekStatementSeparatorOrEof(StatementSeparator2)
}

struct PeekStatementSeparatorOrEof<P>(P);

impl<P> HasOutput for PeekStatementSeparatorOrEof<P> {
    type Output = ();
}

impl<P> Parser for PeekStatementSeparatorOrEof<P>
where
    P: TokenPredicate,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<()>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                let found_it = self.0.test(&token);
                tokenizer.unread(token);
                Ok(if found_it { Some(()) } else { None })
            }
            None => Ok(Some(())),
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
