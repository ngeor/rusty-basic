/// specific module contains implementation that mirrors the base module
/// but it is specific to QBasic

use crate::parser::base::recognizers::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType {
    Unknown,
    Eol,
    WhiteSpace,
    Digits,
    LParen,
    RParen,
    Comma,
    SingleQuote,
    DoubleQuote,
    Dot,
    Equals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    ExclamationMark,
    Pound,
    DollarSign,
    Percent,
    Keyword,
    Identifier
}

pub fn create_recognizers() -> Vec<Box<dyn Recognizer>> {
    vec![
        Box::new(any_recognizer()),
        Box::new(new_line_recognizer()),
        Box::new(white_space_recognizer()),
        Box::new(digits_recognizer()),
    ]
}
