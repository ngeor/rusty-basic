use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Location, QError};
use crate::parser::base::parsers::*;
use crate::parser::base::readers::{file_char_reader, string_char_reader};
use crate::parser::base::recognizers::*;
use crate::parser::base::tokenizers::*;
use crate::parser::expression::expression_node_p;
use crate::parser::{
    Expression, ExpressionNode, ExpressionNodes, Keyword, Statement, SORTED_KEYWORDS_STR,
};
use std::fs::File;
use std::str::Chars;

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
    OctDigits,
    HexDigits,
}

#[derive(Clone, Copy)]
enum OctOrHex {
    Oct,
    Hex,
}

impl From<OctOrHex> for char {
    fn from(value: OctOrHex) -> Self {
        match value {
            OctOrHex::Oct => 'O',
            OctOrHex::Hex => 'H',
        }
    }
}

impl OctOrHex {
    fn is_digit(&self, ch: char) -> bool {
        match self {
            Self::Oct => ch >= '0' && ch <= '7',
            Self::Hex => {
                (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')
            }
        }
    }
}

struct OctHexDigitsRecognizer {
    mode: OctOrHex,
}

impl Recognizer for OctHexDigitsRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        let mut chars = buffer.chars();
        match chars.next() {
            Some('&') => self.after_ampersand(&mut chars),
            _ => Recognition::Negative,
        }
    }
}

impl OctHexDigitsRecognizer {
    fn after_ampersand(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => {
                let needle: char = self.mode.into();
                if ch == needle {
                    self.after_radix(chars)
                } else {
                    Recognition::Negative
                }
            }
            None => Recognition::Partial,
        }
    }

    fn after_radix(&self, chars: &mut Chars) -> Recognition {
        // might be a negative sign, which will lead into Overflow,
        // but needs to be recognized anyway
        match chars.next() {
            Some(ch) => {
                if ch == '-' {
                    self.after_minus(chars)
                } else {
                    self.first_possible_digit(chars, ch)
                }
            }
            None => Recognition::Partial,
        }
    }

    fn after_minus(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => self.first_possible_digit(chars, ch),
            None => Recognition::Partial,
        }
    }

    fn first_possible_digit(&self, chars: &mut Chars, first: char) -> Recognition {
        if self.mode.is_digit(first) {
            self.next_possible_digit(chars)
        } else {
            Recognition::Negative
        }
    }

    fn next_possible_digit(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => {
                if self.mode.is_digit(ch) {
                    self.next_possible_digit(chars)
                } else {
                    Recognition::Negative
                }
            }
            None => Recognition::Positive,
        }
    }
}

pub fn create_recognizers() -> Vec<Box<dyn Recognizer>> {
    vec![
        Box::new(any_single_char_recognizer()),
        Box::new(single_new_line_recognizer()),
        Box::new(many_white_space_recognizer()),
        Box::new(many_digits_recognizer()),
        Box::new(single_char_recognizer('(')),
        Box::new(single_char_recognizer(')')),
        Box::new(single_char_recognizer(':')),
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
        Box::new(keyword_recognizer(&SORTED_KEYWORDS_STR)),
        Box::new(leading_remaining_recognizer(is_letter, |ch| {
            is_letter(ch) || is_digit(ch)
        })),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Oct,
        }),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Hex,
        }),
    ]
}

pub fn create_file_tokenizer(input: File) -> impl Tokenizer {
    create_tokenizer(file_char_reader(input), create_recognizers())
}

#[cfg(test)]
pub fn create_string_tokenizer(input: &str) -> impl Tokenizer {
    create_tokenizer(string_char_reader(input), create_recognizers())
}

// TODO rename to keyword_opt
pub fn keyword_p(keyword: Keyword) -> impl Parser {
    filter_token(|token| {
        Ok(token.kind == TokenType::Keyword as i32 && token.text == keyword.as_str())
    })
}

pub fn keyword(keyword: Keyword) -> impl Parser {
    filter_token(|token| {
        if token.kind == TokenType::Keyword as i32 && token.text == keyword.as_str() {
            Ok(true)
        } else {
            Err(QError::SyntaxError(format!("Expected keyword {}", keyword)))
        }
    })
}

// TODO deprecate this
pub fn keyword_followed_by_whitespace_p(keyword: Keyword) -> impl Parser {
    and(keyword_p(keyword), whitespace())
}

// TODO deprecate this
pub fn item_p(ch: char) -> impl Parser {
    filter_token_by_kind_opt(match ch {
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
}

/// Parses built-in subs with optional arguments
pub fn parse_built_in_sub_with_opt_args(
    keyword: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(keyword)
        .and_opt(expression_node_p().csv_allow_missing())
        .keep_right()
        .map(move |opt_args| {
            Statement::BuiltInSubCall(built_in_sub, map_opt_args_to_flags(opt_args))
        })
}

/// Maps optional arguments to arguments, inserting a dummy first argument indicating
/// which arguments were present in the form of a bit mask.
fn map_opt_args_to_flags(args: Option<Vec<Option<ExpressionNode>>>) -> ExpressionNodes {
    let mut result: ExpressionNodes = vec![];
    let mut mask = 1;
    let mut flags = 0;
    if let Some(args) = args {
        for arg in args {
            if let Some(arg) = arg {
                flags |= mask;
                result.push(arg);
            }
            mask <<= 1;
        }
    }
    result.insert(0, Expression::IntegerLiteral(flags).at(Location::start()));
    result
}

pub fn keyword_pair_p(first: Keyword, second: Keyword) -> impl Parser {
    seq3(keyword_p(first), whitespace(), keyword(second))
}

pub fn whitespace() -> impl Parser {
    filter_token_by_kind(TokenType::WhiteSpace, "Expected whitespace")
}

// TODO rename to whitespace_opt
pub fn whitespace_p() -> impl Parser {
    filter_token_by_kind_opt(TokenType::WhiteSpace)
}

pub fn in_parenthesis<P: Parser>(parser: P) -> impl Parser<Output = P::Output> {
    map(
        seq3(
            filter_token_by_kind(TokenType::LParen, "Expected ("),
            parser,
            filter_token_by_kind(TokenType::RParen, "Expected )"),
        ),
        |(_, output, _)| output,
    )
}
