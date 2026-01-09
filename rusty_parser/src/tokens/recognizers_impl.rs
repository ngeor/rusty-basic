use rusty_pc::*;

use crate::Keyword;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::tokens::string_parsers::CharToStringParser;
use crate::tokens::to_specific_parser::ToSpecificParser;
use crate::tokens::{TokenType, char_parsers};

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
    "\r\n".to_specific_parser().to_token(TokenType::Eol)
}

fn greater_or_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    ">=".to_specific_parser().to_token(TokenType::GreaterEquals)
}

fn less_or_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    "<=".to_specific_parser().to_token(TokenType::LessEquals)
}

fn not_equal() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    "<>".to_specific_parser().to_token(TokenType::NotEquals)
}

fn cr() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific(TokenType::Eol, '\r')
}

fn lf() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    specific(TokenType::Eol, '\n')
}

fn whitespace() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(TokenType::Whitespace, |ch| *ch == ' ' || *ch == '\t')
}

fn digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(TokenType::Digits, char::is_ascii_digit)
}

fn keyword() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // using is_ascii_alphanumeric to read e.g. Sub1 and determine it is not a keyword
    // TODO can be done in a different way e.g. read alphabetic and then ensure it's followed by something other than alphanumeric
    many(TokenType::Keyword, char::is_ascii_alphanumeric)
        .filter(|token| Keyword::try_from(token.as_str()).is_ok())
}

fn identifier() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // TODO leading-remaining
    many(TokenType::Identifier, |ch| {
        *ch == '_' || ch.is_ascii_alphanumeric()
    })
}

fn many<F>(
    token_type: TokenType,
    predicate: F,
) -> impl Parser<RcStringView, Output = Token, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    char_parsers::filter_or_err(
        predicate,
        ParseError::SyntaxError(format!("Expected: {}", token_type)),
    )
    .many_to_str()
    .to_token(token_type)
}

fn specific(
    token_type: TokenType,
    needle: char,
) -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    needle
        .to_specific_parser()
        .one_to_str()
        .to_token(token_type)
}

fn any_symbol() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    char_parsers::AnyChar
        .one_to_str()
        .to_token(TokenType::Symbol)
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
    prefix
        .to_specific_parser()
        .and(
            '-'.to_specific_parser().one_to_str().to_option().and(
                char_parsers::filter_or_err(
                    predicate,
                    ParseError::SyntaxError(format!("Expected: {}", token_type)),
                )
                .many_to_str(),
                StringCombiner,
            ),
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
