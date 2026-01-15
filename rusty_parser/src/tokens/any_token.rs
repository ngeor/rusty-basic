use rusty_pc::*;

use crate::Keyword;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::WithExpected;
use crate::tokens::TokenType;
use crate::tokens::any_char::{AnyChar, AnyCharOrEof};
use crate::tokens::any_symbol::any_symbol;
use crate::tokens::string_parsers::*;

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

/// Parses any number of whitespace characters,
/// and returns them as a single token.
///
/// This is one of the few functions that are public from this module,
/// allowing users to call it bypassing the `any_token` function,
/// if they want to. As whitespace isn't part of other tokens,
/// it should be safe to do so.
pub fn whitespace() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(is_whitespace)
        .to_token(TokenType::Whitespace)
        .with_expected_message("Expected: whitespace")
}

/// Parses any number of whitespace characters, but ignores the result.
/// Whitespace is often ignored, so this function optimizes as it doesn't
/// create a token or store the whitespace characters into a [String].
pub fn whitespace_ignoring() -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    many_ignoring(is_whitespace).with_expected_message("Expected: whitespace")
}

fn is_whitespace(ch: &char) -> bool {
    *ch == ' ' || *ch == '\t'
}

fn digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(char::is_ascii_digit).to_token(TokenType::Digits)
}

fn keyword() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    many(char::is_ascii_alphabetic)
        .filter(|text| Keyword::try_from(text.as_str()).is_ok())
        .to_token(TokenType::Keyword)
        .and_keep_left(ensure_no_illegal_char_after_keyword())
}

fn ensure_no_illegal_char_after_keyword() -> impl Parser<RcStringView, Error = ParseError> {
    PeekParser::new(AnyCharOrEof.filter(is_allowed_char_after_keyword))
}

/// Checks if the given character is illegal after a keyword,
/// which would cause the keyword to be interpreted as an
/// identifier instead.
///
/// Examples of valid identifiers that begin with a keyword:
///
/// - `DIM DIM.`
/// - `DIM DIM$`
/// - `DIM DIM12`
///
/// Note that the dollar sign is the only type qualifier for
/// which this is allowed to happen.
///
/// The parameter is '\0' if we encountered EOF, which is allowed.
fn is_allowed_char_after_keyword(char_or_eof: &char) -> bool {
    *char_or_eof != '.' && *char_or_eof != '$' && !char_or_eof.is_ascii_digit()
}

// TODO validate the max length in `zero_or_more` e.g. `between(0, MAX_LENGTH - 1)`
const MAX_LENGTH: usize = 40;

fn identifier() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    AnyChar
        .filter(char::is_ascii_alphabetic)
        .and(
            AnyChar.filter(is_allowed_char_in_identifier).zero_or_more(),
            StringCombiner,
        )
        .flat_map(|i, s| {
            // TODO add a `.validate()` parser combinator that does not use the input and uses the output only by ref
            if s.len() > MAX_LENGTH {
                Err((true, i, ParseError::IdentifierTooLong))
            } else {
                Ok((i, s))
            }
        })
        .to_token(TokenType::Identifier)
}

fn is_allowed_char_in_identifier(ch: &char) -> bool {
    ch.is_ascii_alphanumeric() || *ch == '.'
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::create_string_tokenizer;

    fn parse_token(input: &str) -> Token {
        any_token()
            .parse(create_string_tokenizer(input.to_owned()))
            .ok()
            .unwrap()
            .1
    }

    #[test]
    fn test_keyword_eof() {
        let input = "DIM";
        let token = parse_token(input);
        assert_eq!(token.kind(), TokenType::Keyword.get_index());
        assert_eq!(token.as_str(), "DIM");
    }

    #[test]
    fn test_keyword_spaces() {
        let input = "DIM ";
        let token = parse_token(input);
        assert_eq!(token.kind(), TokenType::Keyword.get_index());
        assert_eq!(token.as_str(), "DIM");
    }

    #[test]
    fn test_keyword_dollar_sign() {
        let input = "STRING$";
        let token = parse_token(input);
        assert_eq!(token.kind(), TokenType::Identifier.get_index());
        assert_eq!(token.as_str(), "STRING");
    }
}
