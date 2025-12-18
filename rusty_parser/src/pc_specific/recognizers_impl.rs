use rusty_common::Position;

use crate::pc::*;
use crate::pc_ng::Or;
use crate::pc_ng::Parser;
use crate::pc_specific::recognizers_impl::string_parsers::CharToStringParser;
use crate::pc_specific::recognizers_impl::token_parsers::StringToTokenParser;
use crate::pc_specific::TokenType;
use crate::ParseError;
use crate::SORTED_KEYWORDS_STR;
use std::fs::File;

// TODO keyword --> ensure not followed by dollar sign
// TODO make identifier recognizer without dot

pub fn create_file_tokenizer(input: File) -> Result<impl Tokenizer, std::io::Error> {
    let rc_string_view: RcStringView = input.try_into()?;
    Ok(TokenizerParserAdapter::new(rc_string_view, token_parser()))
}

pub fn create_string_tokenizer(input: String) -> impl Tokenizer {
    TokenizerParserAdapter::new(input.into(), token_parser())
}

struct TokenizerParserAdapter<P> {
    readers: Vec<RcStringView>,
    parser: P,
    eof_pos: Position,
}

impl<P> TokenizerParserAdapter<P> {
    fn new(reader: RcStringView, parser: P) -> Self {
        Self {
            readers: vec![reader],
            parser,
            eof_pos: Position::zero(),
        }
    }
}

// TODO remove the fully qualified type names `crate::pc_ng::*`

impl<P> Tokenizer for TokenizerParserAdapter<P>
where
    P: crate::pc_ng::Parser<Input = RcStringView, Output = Token, Error = ParseError>,
{
    fn read(&mut self) -> Option<Token> {
        let reader = self.readers.last().unwrap().clone();
        // TODO this is a temporary fix
        if !reader.is_eof() {
            self.eof_pos = reader.row_col();
            self.eof_pos.inc_col();
        }
        match self.parser.parse(reader) {
            crate::pc_ng::ParseResult::Ok(remaining, token) => {
                self.readers.push(remaining);
                Some(token)
            }
            crate::pc_ng::ParseResult::None(_) | crate::pc_ng::ParseResult::Expected(_, _) => None,
            crate::pc_ng::ParseResult::Err(_, err) => {
                panic!("temporary error in new tokenizer {:?}", err);
            }
        }
    }

    fn unread(&mut self) {
        self.readers.pop().unwrap();
    }

    fn position(&self) -> Position {
        let rc_string_view = self.readers.last().unwrap();
        if rc_string_view.position() < rc_string_view.len() {
            rc_string_view.row_col()
        } else {
            // TODO this is a temporary fix
            self.eof_pos
        }
    }
}

fn token_parser(
) -> impl crate::pc_ng::Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    crate::pc_ng::Or::new(vec![
        // Eol,
        Box::new(eol()),
        // Whitespace: "whitespace",
        Box::new(whitespace()),
        // Digits,
        Box::new(digits()),
        // // keyword needs to be before Identifier, because the first one wins
        // Keyword,
        Box::new(keyword()),
        // // Starts with letter, continues with letters or digits.
        // Identifier,
        Box::new(identifier()),
        // OctDigits,
        Box::new(oct_digits()),
        // HexDigits
        Box::new(hex_digits()),
        // GreaterEquals,
        Box::new(greater_or_equal()),
        // LessEquals,
        Box::new(less_or_equal()),
        // NotEquals,
        Box::new(not_equal()),
        // LParen: '(',
        Box::new(specific(TokenType::LParen, '(')),
        // RParen: ')',
        Box::new(specific(TokenType::RParen, ')')),
        // Colon,
        Box::new(specific(TokenType::Colon, ':')),
        // Semicolon: ';',
        Box::new(specific(TokenType::Semicolon, ';')),
        // Comma: ',',
        Box::new(specific(TokenType::Comma, ',')),
        // SingleQuote,
        Box::new(specific(TokenType::SingleQuote, '\'')),
        // DoubleQuote,
        Box::new(specific(TokenType::DoubleQuote, '"')),
        // Dot,
        Box::new(specific(TokenType::Dot, '.')),
        // Equals,
        Box::new(specific(TokenType::Equals, '=')),
        // Greater,
        Box::new(specific(TokenType::Greater, '>')),
        // Less,
        Box::new(specific(TokenType::Less, '<')),
        // Plus,
        Box::new(specific(TokenType::Plus, '+')),
        // Minus,
        Box::new(specific(TokenType::Minus, '-')),
        // Star,
        Box::new(specific(TokenType::Star, '*')),
        // Slash,
        Box::new(specific(TokenType::Slash, '/')),
        // Ampersand,
        Box::new(specific(TokenType::Ampersand, '&')),
        // ExclamationMark,
        Box::new(specific(TokenType::ExclamationMark, '!')),
        // Pound,
        Box::new(specific(TokenType::Pound, '#')),
        // DollarSign,
        Box::new(specific(TokenType::DollarSign, '$')),
        // Percent,
        Box::new(specific(TokenType::Percent, '%')),
        // // unknown must be last
        // Unknown,
        Box::new(unknown()),
    ])
}

fn eol() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    Or::new(vec![Box::new(crlf()), Box::new(cr()), Box::new(lf())])
}

fn crlf() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::specific('\r')
        .append_char(char_parsers::specific('\n'))
        .to_token(TokenType::Eol)
}

fn greater_or_equal() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::specific('>')
        .append_char(char_parsers::specific('='))
        .to_token(TokenType::GreaterEquals)
}

fn less_or_equal() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::specific('<')
        .append_char(char_parsers::specific('='))
        .to_token(TokenType::LessEquals)
}

fn not_equal() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::specific('<')
        .append_char(char_parsers::specific('>'))
        .to_token(TokenType::NotEquals)
}

fn cr() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    specific(TokenType::Eol, '\r')
}

fn lf() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    specific(TokenType::Eol, '\n')
}

fn whitespace() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    many(TokenType::Whitespace, |ch| *ch == ' ' || *ch == '\t')
}

fn digits() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    many(TokenType::Digits, char::is_ascii_digit)
}

fn keyword() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    // using is_ascii_alphanumeric to read e.g. Sub1 and determine it is not a keyword
    // TODO can be done in a different way e.g. read alphabetic and then ensure it's followed by something other than alphanumeric
    many(TokenType::Keyword, char::is_ascii_alphanumeric).filter(|token| {
        let text = &token.text;
        for keyword in SORTED_KEYWORDS_STR {
            if keyword.eq_ignore_ascii_case(text) {
                return true;
            }
        }
        false
    })
}

fn identifier() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    // TODO leading-remaining
    many(TokenType::Identifier, |ch| {
        *ch == '_' || ch.is_ascii_alphanumeric()
    })
}

fn many<F>(
    token_type: TokenType,
    predicate: F,
) -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    char_parsers::filter(predicate)
        .concatenate()
        .to_token(token_type)
}

fn specific(
    token_type: TokenType,
    needle: char,
) -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::specific(needle).to_str().to_token(token_type)
}

fn unknown() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    char_parsers::any().to_str().to_token(TokenType::Unknown)
}

fn oct_digits() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    oct_or_hex_digits('O', |ch| *ch >= '0' && *ch <= '7', TokenType::OctDigits)
}

fn hex_digits() -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
    oct_or_hex_digits('H', char::is_ascii_hexdigit, TokenType::HexDigits)
}

fn oct_or_hex_digits<F>(
    radix: char,
    predicate: F,
    token_type: TokenType,
) -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    char_parsers::specific('&')
        .append_char(char_parsers::specific(radix))
        .and(
            char_parsers::specific('-').opt().and(
                char_parsers::filter(predicate).concatenate(),
                |opt_minus, mut digits| {
                    match opt_minus {
                        Some(minus) => {
                            // TODO prevent insert
                            digits.insert(0, minus);
                            digits
                        }
                        _ => digits,
                    }
                },
            ),
            |mut left, right| {
                // TODO prevent push_str
                left.push_str(&right);
                left
            },
        )
        .to_token(token_type)
}

mod token_parsers {
    use super::*;

    struct AnyPos;

    impl crate::pc_ng::Parser for AnyPos {
        type Input = RcStringView;
        type Output = Position;
        type Error = ParseError;

        fn parse(
            &self,
            input: Self::Input,
        ) -> crate::pc_ng::ParseResult<Self::Input, Self::Output, Self::Error> {
            if input.is_eof() {
                crate::pc_ng::ParseResult::None(input)
            } else {
                let pos = input.row_col();
                crate::pc_ng::ParseResult::Ok(input, pos)
            }
        }
    }

    pub trait StringToTokenParser {
        fn to_token(
            self,
            token_type: TokenType,
        ) -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError>;
    }

    impl<P> StringToTokenParser for P
    where
        P: Parser<Input = RcStringView, Output = String, Error = ParseError>,
    {
        fn to_token(
            self,
            token_type: TokenType,
        ) -> impl Parser<Input = RcStringView, Output = Token, Error = ParseError> {
            AnyPos.and(self, move |pos, text| Token {
                kind: token_type.into(),
                text,
                pos,
            })
        }
    }
}

mod string_parsers {
    use super::*;

    pub trait CharToStringParser {
        type Input;
        type Error;

        fn concatenate(
            self,
        ) -> impl Parser<Input = Self::Input, Output = String, Error = Self::Error>;

        fn to_str(self) -> impl Parser<Input = Self::Input, Output = String, Error = Self::Error>;

        fn append_char(
            self,
            other: impl Parser<Input = Self::Input, Output = char, Error = Self::Error>,
        ) -> impl Parser<Input = Self::Input, Output = String, Error = Self::Error>
        where
            Self::Input: Clone;
    }

    impl<P> CharToStringParser for P
    where
        P: Parser<Output = char>,
    {
        type Input = P::Input;
        type Error = P::Error;

        fn concatenate(self) -> impl Parser<Input = P::Input, Output = String, Error = P::Error> {
            self.many(String::from, |mut s, c| {
                s.push(c);
                s
            })
        }

        fn to_str(self) -> impl Parser<Input = P::Input, Output = String, Error = P::Error> {
            self.map(String::from)
        }

        fn append_char(
            self,
            other: impl Parser<Input = Self::Input, Output = char, Error = Self::Error>,
        ) -> impl Parser<Input = Self::Input, Output = String, Error = Self::Error>
        where
            Self::Input: Clone,
        {
            self.and(other, |l, r| {
                let mut s = String::from(l);
                s.push(r);
                s
            })
        }
    }
}

mod char_parsers {
    use super::*;

    pub fn any() -> impl Parser<Input = RcStringView, Output = char, Error = ParseError> {
        AnyChar
    }

    pub fn filter<F>(
        predicate: F,
    ) -> impl Parser<Input = RcStringView, Output = char, Error = ParseError>
    where
        F: Fn(&char) -> bool,
    {
        any().filter(predicate)
    }

    pub fn specific(
        needle: char,
    ) -> impl Parser<Input = RcStringView, Output = char, Error = ParseError> {
        filter(move |ch| *ch == needle)
    }

    struct AnyChar;

    impl crate::pc_ng::Parser for AnyChar {
        type Input = RcStringView;
        type Output = char;
        type Error = ParseError;

        fn parse(
            &self,
            input: Self::Input,
        ) -> crate::pc_ng::ParseResult<Self::Input, Self::Output, Self::Error> {
            if input.is_eof() {
                crate::pc_ng::ParseResult::None(input)
            } else {
                let ch = input.char();
                crate::pc_ng::ParseResult::Ok(input.inc_position(), ch)
            }
        }
    }
}
