//
// TokenKindParser
//

use std::convert::TryFrom;
use crate::parser::pc::{ErrorProvider, OptAndPC, Parser, Token, TokenPredicate, TokenPredicateParser};
use crate::parser::pc_specific::{TokenType, whitespace};

pub struct TokenKindParser {
    token_type: TokenType,
}

impl TokenKindParser {
    pub fn new(token_type: TokenType) -> Self {
        Self { token_type }
    }
}

impl TokenPredicate for TokenKindParser {
    fn test(&self, token: &Token) -> bool {
        token.kind == self.token_type as i32
    }
}

impl ErrorProvider for TokenKindParser {
    fn provide_error_message(&self) -> String {
        match char::try_from(self.token_type) {
            Ok(ch) => format!("Expected: {}", ch),
            _ => {
                if self.token_type == TokenType::Whitespace {
                    "Expected: whitespace".to_owned()
                } else {
                    // TODO use Display instead of Debug
                    format!("Expected: token of type {:?}", self.token_type)
                }
            }
        }
    }
}

// TODO #[deprecated]
pub fn item_p(ch: char) -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser::new(match ch {
        ',' => TokenType::Comma,
        '=' => TokenType::Equals,
        '$' => TokenType::DollarSign,
        '\'' => TokenType::SingleQuote,
        '-' => TokenType::Minus,
        '*' => TokenType::Star,
        '#' => TokenType::Pound,
        '.' => TokenType::Dot,
        ';' => TokenType::Semicolon,
        '>' => TokenType::Greater,
        '<' => TokenType::Less,
        ':' => TokenType::Colon,
        _ => panic!("not implemented {}", ch),
    })
        .parser()
}

/// Equal sign, surrounded by optional whitespace.
///
/// `<ws>? = <ws>?`
pub fn equal_sign() -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), item_p('=')).and_opt(whitespace())
        .map(|((_, t), _)| t)
}

/// Comma, surrounded by optional whitespace.
///
/// `<ws>? , <ws>?`
pub fn comma() -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), item_p(',')).and_opt(whitespace())
        .map(|((_, t), _)| t)
}

/// Star (*), surrounded by optional whitespace.
///
/// `<ws>? * <ws>?`
pub fn star() -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), item_p('*')).and_opt(whitespace())
        .map(|((_, t), _)| t)
}

/// Colon, surrounded by optional whitespace.
///
/// `<ws>? : <ws>?`
pub fn colon() -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), item_p(':')).and_opt(whitespace())
        .map(|((_, t), _)| t)
}
