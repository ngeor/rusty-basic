use crate::parser::char_reader::file_char_reader;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use crate::parser::SORTED_KEYWORDS_STR;
use crate::recognizers;
use std::fs::File;
use std::str::Chars;

// TODO keyword --> ensure not followed by dollar sign
// TODO make identifier recognizer without dot

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
            Self::Hex => is_digit(ch) || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F'),
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

pub fn create_recognizers() -> RecognizersWithType {
    recognizers![
        TokenType::Eol => single_new_line_recognizer,
        TokenType::Whitespace => many_white_space_recognizer(),
        TokenType::Digits => many_digits_recognizer(),
        TokenType::LParen => '(',
        TokenType::RParen => ')',
        TokenType::Colon => ':',
        TokenType::Semicolon => ';',
        TokenType::Comma => ',',
        TokenType::SingleQuote => '\'',
        TokenType::DoubleQuote => '"',
        TokenType::Dot => '.',
        TokenType::Equals => '=',
        TokenType::Greater => '>',
        TokenType::Less => '<',
        TokenType::GreaterEquals => ">=",
        TokenType::LessEquals => "<=",
        TokenType::NotEquals => "<>",
        TokenType::Plus => '+',
        TokenType::Minus => '-',
        TokenType::Star => '*',
        TokenType::Slash => '/',
        TokenType::Ampersand => '&',
        TokenType::ExclamationMark => '!',
        TokenType::Pound => '#',
        TokenType::DollarSign => '$',
        TokenType::Percent => '%',
        TokenType::Keyword => keyword_recognizer(SORTED_KEYWORDS_STR),
        TokenType::Identifier => leading_remaining_recognizer(is_letter, |ch| {
            is_letter(ch) || is_digit(ch) || ch == '.'
        }),
        TokenType::OctDigits => OctHexDigitsRecognizer {
            mode: OctOrHex::Oct,
        },
        TokenType::HexDigits => OctHexDigitsRecognizer {
            mode: OctOrHex::Hex,
        },
        TokenType::Unknown => any_single_char_recognizer,
    ]
}

pub fn create_file_tokenizer(input: File) -> impl Tokenizer {
    create_tokenizer(file_char_reader(input), create_recognizers())
}

#[cfg(test)]
pub mod test_helper {
    use crate::parser::char_reader::test_helper::string_char_reader;
    use crate::parser::pc::{create_tokenizer, Tokenizer};
    use crate::parser::pc_specific::create_recognizers;

    pub fn create_string_tokenizer<T>(input: T) -> impl Tokenizer
    where
        T: AsRef<[u8]>,
    {
        create_tokenizer(string_char_reader(input), create_recognizers())
    }
}
