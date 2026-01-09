use rusty_pc::*;

use crate::Keyword;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::tokens::TokenType;
use crate::tokens::any_symbol::any_symbol;
use crate::tokens::string_parsers::*;

// TODO make identifier recognizer without dot

/// Parses any token.
pub fn any_token() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
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
        // Symbol must be last,
        Box::new(any_symbol()),
    ])
}

/// Peeks the next token without consuming it.
pub fn peek_token() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    PeekParser::new(any_token())
}

fn eol() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    OrParser::new(vec![Box::new(crlf()), Box::new(cr()), Box::new(lf())])
}

fn crlf() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific("\r\n").to_token(TokenType::Eol)
}

fn greater_or_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific(">=").to_token(TokenType::GreaterEquals)
}

fn less_or_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific("<=").to_token(TokenType::LessEquals)
}

fn not_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific("<>").to_token(TokenType::NotEquals)
}

fn cr() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    one('\r').to_token(TokenType::Eol)
}

fn lf() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    one('\n').to_token(TokenType::Eol)
}

fn whitespace() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(|ch| *ch == ' ' || *ch == '\t').to_token(TokenType::Whitespace)
}

fn digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(char::is_ascii_digit).to_token(TokenType::Digits)
}

fn keyword() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // using is_ascii_alphanumeric to read e.g. Sub1 and determine it is not a keyword
    // TODO can be done in a different way e.g. read alphabetic and then ensure it's followed by something other than alphanumeric
    many(char::is_ascii_alphanumeric)
        .filter(|text| Keyword::try_from(text.as_str()).is_ok())
        .to_token(TokenType::Keyword)
}

fn identifier() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // TODO leading-remaining
    many(|ch| *ch == '_' || ch.is_ascii_alphanumeric()).to_token(TokenType::Identifier)
}

fn oct_digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    oct_or_hex_digits('O', |ch| *ch >= '0' && *ch <= '7', TokenType::OctDigits)
}

fn hex_digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    oct_or_hex_digits('H', char::is_ascii_hexdigit, TokenType::HexDigits)
}

fn oct_or_hex_digits<F>(
    radix: char,
    predicate: F,
    token_type: TokenType,
) -> impl Parser<RcStringView, Output = Token, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    let prefix = format!("&{}", radix);
    specific_owned(prefix)
        .and(
            one('-').to_option().and(many(predicate), StringCombiner),
            StringCombiner,
        )
        .to_token(token_type)
}

trait StringToTokenParser {
    fn to_token(
        self,
        token_type: TokenType,
    ) -> impl Parser<RcStringView, Output = Token, Error = ParseError>;
}

impl<P> StringToTokenParser for P
where
    P: Parser<RcStringView, Output = String, Error = ParseError>,
{
    fn to_token(
        self,
        token_type: TokenType,
    ) -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
        self.map(move |text| Token::new(token_type.get_index(), text))
    }
}
