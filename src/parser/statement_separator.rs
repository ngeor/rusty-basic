use crate::common::QError;
use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, HasOutput, OptAndPC, OrTrait, Parser, TokenPredicate};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::specific::{whitespace, TokenKindParser, TokenType, eol_or_whitespace, OrSyntaxErrorTrait};

// TODO convert to NonOptParser
pub fn comment_separator() -> impl Parser<Output = ()> {
    OptAndPC::new(whitespace(), eol_or_whitespace()).fn_map(|_| ())
}

// <ws>* '\'' (undoing it)
// <ws>* ':' <ws*>
// <ws>* EOL <ws | eol>*
// TODO convert to NonOptParser
pub fn non_comment_separator() -> impl Parser<Output = ()> {
    OptAndPC::new(whitespace(),
    SingleQuotePeek
        .or(colon_separator_p())
        .or(eol_separator_p())
        .or_syntax_error("Expected: end-of-statement")
    ).fn_map(|_| ())
}

// '\'' (undoing it)
// not adding the ' character in the resulting string because it was already undone
struct SingleQuotePeek;

impl HasOutput for SingleQuotePeek {
    type Output = ();
}

impl Parser for SingleQuotePeek {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if token.kind == TokenType::SingleQuote as i32 {
                    tokenizer.unread(token);
                    Ok(Some(()))
                } else {
                    tokenizer.unread(token);
                    Ok(None)
                }
            }
            None => Ok(None)
        }
    }
}

pub fn peek_eof_or_statement_separator() -> impl Parser<Output = ()> {
    PeekStatementSeparatorOrEof(StatementSeparator2)
}

struct PeekStatementSeparatorOrEof<P>(P);

impl<P> HasOutput for PeekStatementSeparatorOrEof<P> {
    type Output = ();
}

impl<P> Parser for PeekStatementSeparatorOrEof<P> where P : TokenPredicate {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<()>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                let found_it = self.0.test(&token);
                tokenizer.unread(token);
                Ok(if found_it { Some (()) } else { None })
            }
            None => Ok(Some(()))
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

// ':' <ws>*
fn colon_separator_p() -> impl Parser<Output = ()> {
    TokenKindParser::new(TokenType::Colon).parser()
        .and_opt(whitespace())
        .fn_map(|_| ())

}

// <eol> < ws | eol >*
// TODO rename to _opt
fn eol_separator_p() -> impl Parser<Output = ()> {
    TokenKindParser::new(TokenType::Eol).parser()
        .and_opt(eol_or_whitespace())
        .fn_map(|_| ())
}
