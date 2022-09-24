use crate::common::QError;
use crate::parser::base::parsers::{FnMapTrait, HasOutput, Parser, TokenPredicate};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::{eol_or_whitespace, TokenType};

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
            Self::Comment => comment_separator().parse(tokenizer),
            Self::NonComment => non_comment_separator().parse(tokenizer),
        }
    }
}

// TODO convert to NonOptParser
fn comment_separator() -> impl Parser<Output = ()> {
    eol_or_whitespace().preceded_by_opt_ws().fn_map(|_| ())
}

// <ws>* '\'' (undoing it)
// <ws>* ':' <ws*>
// <ws>* EOL <ws | eol>*
// TODO convert to NonOptParser
fn non_comment_separator() -> impl Parser<Output = ()> {
    CommonSeparator
}

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
