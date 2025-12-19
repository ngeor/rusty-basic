use rusty_common::Position;

use crate::pc::*;
use crate::pc_specific::recognizers_impl::string_parsers::CharToStringParser;
use crate::pc_specific::recognizers_impl::token_parsers::StringToTokenParser;
use crate::pc_specific::TokenType;
use crate::ParseError;
use crate::SORTED_KEYWORDS_STR;
use std::fs::File;

// TODO keyword --> ensure not followed by dollar sign
// TODO make identifier recognizer without dot

pub fn create_file_tokenizer(input: File) -> Result<RcStringView, std::io::Error> {
    let rc_string_view: RcStringView = input.try_into()?;
    Ok(rc_string_view)
}

pub fn create_string_tokenizer(input: String) -> RcStringView {
    input.into()
}

pub fn token_parser() -> impl Parser<RcStringView, Output = Token> {
    OrParser::new(vec![
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

fn eol() -> impl Parser<RcStringView, Output = Token> {
    OrParser::new(vec![Box::new(crlf()), Box::new(cr()), Box::new(lf())])
}

fn crlf() -> impl Parser<RcStringView, Output = Token> {
    char_parsers::specific('\r')
        .append_char(char_parsers::specific('\n'))
        .to_token(TokenType::Eol)
}

fn greater_or_equal() -> impl Parser<RcStringView, Output = Token> {
    char_parsers::specific('>')
        .append_char(char_parsers::specific('='))
        .to_token(TokenType::GreaterEquals)
}

fn less_or_equal() -> impl Parser<RcStringView, Output = Token> {
    char_parsers::specific('<')
        .append_char(char_parsers::specific('='))
        .to_token(TokenType::LessEquals)
}

fn not_equal() -> impl Parser<RcStringView, Output = Token> {
    char_parsers::specific('<')
        .append_char(char_parsers::specific('>'))
        .to_token(TokenType::NotEquals)
}

fn cr() -> impl Parser<RcStringView, Output = Token> {
    specific(TokenType::Eol, '\r')
}

fn lf() -> impl Parser<RcStringView, Output = Token> {
    specific(TokenType::Eol, '\n')
}

fn whitespace() -> impl Parser<RcStringView, Output = Token> {
    many(TokenType::Whitespace, |ch| *ch == ' ' || *ch == '\t')
}

fn digits() -> impl Parser<RcStringView, Output = Token> {
    many(TokenType::Digits, char::is_ascii_digit)
}

fn keyword() -> impl Parser<RcStringView, Output = Token> {
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

fn identifier() -> impl Parser<RcStringView, Output = Token> {
    // TODO leading-remaining
    many(TokenType::Identifier, |ch| {
        *ch == '_' || ch.is_ascii_alphanumeric()
    })
}

fn many<F>(token_type: TokenType, predicate: F) -> impl Parser<RcStringView, Output = Token>
where
    F: Fn(&char) -> bool,
{
    char_parsers::filter(predicate)
        .concatenate()
        .to_token(token_type)
}

fn specific(token_type: TokenType, needle: char) -> impl Parser<RcStringView, Output = Token> {
    char_parsers::specific(needle).to_str().to_token(token_type)
}

fn unknown() -> impl Parser<RcStringView, Output = Token> {
    char_parsers::any().to_str().to_token(TokenType::Unknown)
}

fn oct_digits() -> impl Parser<RcStringView, Output = Token> {
    oct_or_hex_digits('O', |ch| *ch >= '0' && *ch <= '7', TokenType::OctDigits)
}

fn hex_digits() -> impl Parser<RcStringView, Output = Token> {
    oct_or_hex_digits('H', char::is_ascii_hexdigit, TokenType::HexDigits)
}

fn oct_or_hex_digits<F>(
    radix: char,
    predicate: F,
    token_type: TokenType,
) -> impl Parser<RcStringView, Output = Token>
where
    F: Fn(&char) -> bool,
{
    char_parsers::specific('&')
        .append_char(char_parsers::specific(radix))
        .and(
            char_parsers::specific('-').to_option().and(
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

    impl Parser<RcStringView> for AnyPos {
        type Output = Position;

        fn parse(
            &self,
            input: RcStringView,
        ) -> ParseResult<RcStringView, Self::Output, ParseError> {
            if input.is_eof() {
                default_parse_error(input)
            } else {
                let pos = input.position();
                Ok((input, pos))
            }
        }
    }

    pub trait StringToTokenParser {
        fn to_token(self, token_type: TokenType) -> impl Parser<RcStringView, Output = Token>;
    }

    impl<P> StringToTokenParser for P
    where
        P: Parser<RcStringView, Output = String>,
    {
        fn to_token(self, token_type: TokenType) -> impl Parser<RcStringView, Output = Token> {
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

    pub trait CharToStringParser<I> {
        fn concatenate(self) -> impl Parser<I, Output = String>;

        fn to_str(self) -> impl Parser<I, Output = String>;

        fn append_char(
            self,
            other: impl Parser<I, Output = char>,
        ) -> impl Parser<I, Output = String>
        where
            I: Clone;
    }

    impl<I, P> CharToStringParser<I> for P
    where
        I: Clone,
        P: Parser<I, Output = char>,
    {
        fn concatenate(self) -> impl Parser<I, Output = String> {
            Many::new(self, String::from, |mut s: String, c| {
                s.push(c);
                s
            })
        }

        fn to_str(self) -> impl Parser<I, Output = String> {
            self.map(String::from)
        }

        fn append_char(
            self,
            other: impl Parser<I, Output = char>,
        ) -> impl Parser<I, Output = String> {
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

    pub fn any() -> impl Parser<RcStringView, Output = char> {
        AnyChar
    }

    pub fn filter<F>(predicate: F) -> impl Parser<RcStringView, Output = char>
    where
        F: Fn(&char) -> bool,
    {
        any().filter(predicate)
    }

    pub fn specific(needle: char) -> impl Parser<RcStringView, Output = char> {
        filter(move |ch| *ch == needle)
    }

    struct AnyChar;

    impl Parser<RcStringView> for AnyChar {
        type Output = char;

        fn parse(
            &self,
            input: RcStringView,
        ) -> ParseResult<RcStringView, Self::Output, ParseError> {
            if input.is_eof() {
                default_parse_error(input)
            } else {
                let ch = input.char();
                Ok((input.inc_position(), ch))
            }
        }
    }
}
