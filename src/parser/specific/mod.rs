use std::fs::File;
use crate::parser::base::parsers::*;
use crate::parser::base::recognizers::*;
use crate::parser::base::tokenizers::*;
use crate::parser::Statement;
use std::str::Chars;
use crate::parser::base::readers::{file_char_reader, string_char_reader};

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

#[derive(Copy)]
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
        Box::new(keyword_recognizer(&KEYWORDS)),
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
    create_tokenizer(
        file_char_reader(input),
        create_recognizers()
    )
}

#[cfg(test)]
pub fn create_string_tokenizer(input: &str) -> impl Tokenizer {
    create_tokenizer(
        string_char_reader(input),
        create_recognizers()
    )
}
