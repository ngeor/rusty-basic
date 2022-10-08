use crate::common::QError;
use crate::parser::pc::TokenKind;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType {
    Eol,
    Whitespace,
    Digits,
    LParen,
    RParen,
    Colon,
    Semicolon,
    Comma,
    SingleQuote,
    DoubleQuote,
    Dot,
    Equals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    NotEquals,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    ExclamationMark,
    Pound,
    DollarSign,
    Percent,
    // keyword needs to be before Identifier
    Keyword,
    /// Starts with letter, continues with letters, digits or dots.
    Identifier,
    OctDigits,
    HexDigits,

    // unknown must be last
    Unknown,
}

impl From<TokenKind> for TokenType {
    fn from(value: TokenKind) -> Self {
        let all_tokens = [
            TokenType::Eol,
            TokenType::Whitespace,
            TokenType::Digits,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::Colon,
            TokenType::Semicolon,
            TokenType::Comma,
            TokenType::SingleQuote,
            TokenType::DoubleQuote,
            TokenType::Dot,
            TokenType::Equals,
            TokenType::Greater,
            TokenType::Less,
            TokenType::GreaterEquals,
            TokenType::LessEquals,
            TokenType::NotEquals,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::Ampersand,
            TokenType::ExclamationMark,
            TokenType::Pound,
            TokenType::DollarSign,
            TokenType::Percent,
            TokenType::Keyword,
            TokenType::Identifier,
            TokenType::OctDigits,
            TokenType::HexDigits,
            TokenType::Unknown,
        ];
        all_tokens[value as usize]
    }
}

impl TryFrom<TokenType> for char {
    type Error = QError;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::Semicolon => Ok(';'),
            TokenType::Comma => Ok(','),
            TokenType::Equals => Ok('='),
            TokenType::Colon => Ok(':'),
            TokenType::Star => Ok('*'),
            TokenType::LParen => Ok('('),
            TokenType::RParen => Ok(')'),
            _ => Err(QError::InternalError(format!(
                "not implemented {:?}",
                value
            ))),
        }
    }
}

impl From<TokenType> for QError {
    fn from(token_type: TokenType) -> Self {
        QError::Expected(match char::try_from(token_type) {
            Ok(ch) => {
                format!("Expected: {}", ch)
            }
            _ => match token_type {
                TokenType::Whitespace => "Expected: whitespace".to_owned(),
                // TODO : implement and use Display
                _ => format!("Expected: token of type {:?}", token_type),
            },
        })
    }
}
