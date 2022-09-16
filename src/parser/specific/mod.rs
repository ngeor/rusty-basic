use crate::parser::base::parsers::*;
use crate::parser::base::recognizers::*;
use crate::parser::base::tokenizers::*;
use crate::parser::pc::{is_digit, is_letter};
use crate::parser::Statement;

/// specific module contains implementation that mirrors the base module
/// but it is specific to QBasic
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
    Identifier,
}

const KEYWORDS: [&str; 4] = ["DIM", "GOSUB", "INPUT", "PRINT"];

pub fn create_recognizers() -> Vec<Box<dyn Recognizer>> {
    vec![
        Box::new(any_single_char_recognizer()),
        Box::new(single_new_line_recognizer()),
        Box::new(many_white_space_recognizer()),
        Box::new(many_digits_recognizer()),
        Box::new(single_char_recognizer('(')),
        Box::new(single_char_recognizer(')')),
        Box::new(single_char_recognizer(',')),
        Box::new(single_char_recognizer('\'')),
        Box::new(single_char_recognizer('"')),
        Box::new(single_char_recognizer('.')),
        Box::new(single_char_recognizer('=')),
        Box::new(single_char_recognizer('>')),
        Box::new(single_char_recognizer('<')),
        Box::new(str_recognizer(">=")),
        Box::new(str_recognizer("<=")),
        Box::new(single_char_recognizer('+')),
        Box::new(single_char_recognizer('-')),
        Box::new(single_char_recognizer('*')),
        Box::new(single_char_recognizer('/')),
        Box::new(single_char_recognizer('&')),
        Box::new(single_char_recognizer('!')),
        Box::new(single_char_recognizer('#')),
        Box::new(single_char_recognizer('$')),
        Box::new(single_char_recognizer('%')),
        Box::new(keyword_recognizer(&KEYWORDS)),
        Box::new(leading_remaining_recognizer(is_letter, |ch| {
            is_letter(ch) || is_digit(ch)
        })),
    ]
}

fn go_sub_parser() -> impl Parser {
    // keyword GOSUB + whitespace + bare name (any word without dot)
    map(
        seq3(keyword_opt("GOSUB"), whitespace(), identifier()),
        |(_, _, label_token)| Statement::GoSub(label_token.text.into()),
    )
}

fn keyword_opt(needle: &str) -> impl Parser<Item = Token> + '_ {
    filter_token(move |token| {
        token.kind == TokenType::Keyword as i32 && token.text.eq_ignore_ascii_case(needle)
    })
}

fn whitespace() -> impl Parser<Item = Token> {
    filter_token_by_kind(TokenType::WhiteSpace as i32, "Expected whitespace")
}

fn whitespace_opt() -> impl Parser<Item = Token> {
    filter_token_by_kind_opt(TokenType::WhiteSpace as i32)
}

fn identifier() -> impl Parser<Item = Token> {
    filter_token_by_kind(TokenType::Identifier as i32, "Expected identifier")
}

// fn expression() -> impl Parser {
//     todo!()
// }
//
// fn expression_in_parenthesis() -> impl Parser {
//     todo!()
// }
