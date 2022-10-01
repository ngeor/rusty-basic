use crate::common::QError;
use crate::parser::char_reader::file_char_reader;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::SORTED_KEYWORDS_STR;
use std::fs::File;
use std::str::Chars;

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

pub fn create_recognizers() -> Vec<Box<dyn Recognizer>> {
    vec![
        Box::new(single_new_line_recognizer()),
        Box::new(many_white_space_recognizer()),
        Box::new(many_digits_recognizer()),
        Box::new(single_char_recognizer('(')),
        Box::new(single_char_recognizer(')')),
        Box::new(single_char_recognizer(':')),
        Box::new(single_char_recognizer(';')),
        Box::new(single_char_recognizer(',')),
        Box::new(single_char_recognizer('\'')),
        Box::new(single_char_recognizer('"')),
        Box::new(single_char_recognizer('.')),
        Box::new(single_char_recognizer('=')),
        Box::new(single_char_recognizer('>')),
        Box::new(single_char_recognizer('<')),
        Box::new(str_recognizer(">=")),
        Box::new(str_recognizer("<=")),
        Box::new(str_recognizer("<>")),
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
            is_letter(ch) || is_digit(ch) || ch == '.'
        })),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Oct,
        }),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Hex,
        }),
        Box::new(any_single_char_recognizer()),
    ]
}

pub fn create_file_tokenizer(input: File) -> impl Tokenizer {
    create_tokenizer(file_char_reader(input), create_recognizers())
}

//
// OrError
//

pub struct OrError<P>(P, QError);

impl<P> Parser for OrError<P>
where
    P: Parser,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.0.parse_opt(tokenizer)? {
            Some(value) => Ok(value),
            _ => Err(self.1.clone()),
        }
    }
}

//
// Or Syntax Error
//

pub struct OrSyntaxError<'a, P>(P, &'a str);

impl<'a, P> Parser for OrSyntaxError<'a, P>
where
    P: Parser,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.0.parse_opt(tokenizer)? {
            Some(value) => Ok(value),
            None => Err(QError::syntax_error(self.1)),
        }
    }
}

pub trait OrErrorTrait
where
    Self: Sized,
{
    fn or_error(self, err: QError) -> OrError<Self>;

    fn or_syntax_error(self, msg: &str) -> OrSyntaxError<Self>;
}

impl<S> OrErrorTrait for S {
    fn or_error(self, err: QError) -> OrError<Self> {
        OrError(self, err)
    }

    fn or_syntax_error(self, msg: &str) -> OrSyntaxError<Self> {
        OrSyntaxError(self, msg)
    }
}

pub fn identifier_or_keyword() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| {
            token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32
        })
        .map_incomplete_err(|| QError::Expected("Expected: identifier or keyword".to_owned()))
}

pub fn identifier_or_keyword_without_dot() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| {
            (token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32)
                && !token.text.contains('.')
        })
        .map_incomplete_err(|| QError::Expected("Expected: identifier or keyword".to_owned()))
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
