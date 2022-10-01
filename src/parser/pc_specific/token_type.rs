use crate::common::QError;
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
    Identifier,
    OctDigits,
    HexDigits,

    // unknown must be last
    Unknown,
}

impl TryFrom<i32> for TokenType {
    type Error = QError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
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
        if value >= 0 && value < all_tokens.len() as i32 {
            Ok(all_tokens[value as usize])
        } else {
            Err(QError::InternalError(format!(
                "Token index {} out of bounds",
                value
            )))
        }
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
                _ => format!("Expected: token of type {:?}", token_type),
            },
        })
    }
}
