use rusty_pc::and::StringCombiner;
use rusty_pc::many::{ManyCombiner, StringManyCombiner};
use rusty_pc::text::{many_str, many_str_with_combiner, one_char_to_str};
use rusty_pc::*;

use crate::input::StringView;
use crate::tokens::TokenType;
use crate::tokens::any_symbol::any_symbol;
use crate::{Keyword, ParserError};

/// Parses any token.
pub fn any_token() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    OrParser::new(vec![
        // Eol,
        Box::new(eol()),
        // Whitespace: "whitespace",
        Box::new(whitespace()),
        // Digits,
        Box::new(digits()),
        // // keyword needs to be before Identifier, because the first one wins
        // Keyword,
        Box::new(any_keyword()),
        // // Starts with letter, continues with letters or digits.
        // Identifier,
        Box::new(identifier()),
        // OctDigits,
        Box::new(oct_digits()),
        // HexDigits
        Box::new(hex_digits()),
        // GreaterEquals >=
        // Greater       >
        Box::new(gt_or_ge()),
        // LessEquals    <=
        // NotEquals     <>
        // Less          <
        Box::new(lt_or_le_or_ne()),
        // Equals =
        Box::new(equals()),
        // Symbol must be last,
        Box::new(any_symbol()),
    ])
}

/// Peeks the next token without consuming it.
pub fn peek_token() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    any_token().peek()
}

fn eol() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    OrParser::new(vec![Box::new(cr_or_crlf()), Box::new(lf())])
}

fn cr_or_crlf() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    one_p('\r')
        .and(one_p('\n').to_option(), StringCombiner)
        .to_token(TokenType::Eol)
}

fn lf() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    one_char_to_str('\n').to_token(TokenType::Eol)
}

fn gt_or_ge() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    one_p('>')
        .and(one_p('=').to_option(), StringCombiner)
        .map(|text| {
            if text.len() == 1 {
                Token::new(TokenType::Greater.get_index(), text)
            } else {
                Token::new(TokenType::GreaterEquals.get_index(), text)
            }
        })
}

fn lt_or_le_or_ne() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    one_p('<')
        .and(one_of_p(&['>', '=']).to_option(), StringCombiner)
        .map(|text| {
            if text.len() == 1 {
                Token::new(TokenType::Less.get_index(), text)
            } else if text.ends_with('=') {
                Token::new(TokenType::LessEquals.get_index(), text)
            } else {
                Token::new(TokenType::NotEquals.get_index(), text)
            }
        })
}

fn equals() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    one_char_to_str('=').to_token(TokenType::Equals)
}

/// Parses any number of whitespace characters,
/// and returns them as a single token.
fn whitespace() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    whitespace_collecting(StringManyCombiner).to_token(TokenType::Whitespace)
}

fn whitespace_collecting<C, O>(
    combiner: C,
) -> impl Parser<StringView, Output = O, Error = ParserError>
where
    C: ManyCombiner<char, O>,
    O: Default,
{
    many_str_with_combiner(is_whitespace, combiner).with_expected_message("Expected: whitespace")
}

fn is_whitespace(ch: &char) -> bool {
    *ch == ' ' || *ch == '\t'
}

fn digits() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    many_str(char::is_ascii_digit).to_token(TokenType::Digits)
}

fn any_keyword() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    many_str(char::is_ascii_alphabetic)
        .filter(|text: &String| Keyword::try_from(text.as_str()).is_ok())
        .and_keep_left(ensure_no_illegal_char_after_keyword())
        .with_expected_message("Expected: Keyword")
        .to_token(TokenType::Keyword)
}

fn ensure_no_illegal_char_after_keyword()
-> impl Parser<StringView, Output = (), Error = ParserError> {
    peek_p().to_option().and_then(|opt_ch| match opt_ch {
        Some(ch) if is_allowed_char_after_keyword(ch) => Ok(()),
        None => Ok(()),
        _ => default_parse_error(),
    })
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
fn is_allowed_char_after_keyword(ch: char) -> bool {
    // The is_ascii_alphanumeric used to be is_ascii_digit, to detect numbers.
    // With the introduction of keyword_ignoring, which stops parsing as soon
    // as the keyword is detected and not when we stop detecting letters,
    // it needed to switch to is_ascii_alphanumeric to ignore words that start
    // with keywords, e.g. GetName.
    ch != '.' && ch != '$' && !ch.is_ascii_alphanumeric()
}

const MAX_LENGTH: usize = 40;

fn identifier() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    read_p()
        .filter(char::is_ascii_alphabetic)
        .and(
            read_p()
                .filter(is_allowed_char_in_identifier)
                .zero_or_more(),
            StringCombiner,
        )
        .and_then(|value| {
            if value.len() > MAX_LENGTH {
                Err(ParserError::IdentifierTooLong)
            } else {
                Ok(value)
            }
        })
        .to_token(TokenType::Identifier)
}

fn is_allowed_char_in_identifier(ch: &char) -> bool {
    ch.is_ascii_alphanumeric() || *ch == '.'
}

fn oct_digits() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    oct_or_hex_digits('O', |ch| *ch >= '0' && *ch <= '7', TokenType::OctDigits)
}

fn hex_digits() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    oct_or_hex_digits('H', char::is_ascii_hexdigit, TokenType::HexDigits)
}

fn oct_or_hex_digits<F>(
    radix: char,
    predicate: F,
    token_type: TokenType,
) -> impl Parser<StringView, Output = Token, Error = ParserError>
where
    F: Fn(&char) -> bool,
{
    one_p('&')
        .and(one_p(radix), StringCombiner)
        .and(
            one_char_to_str('-')
                .to_option()
                .and(many_str(predicate), StringCombiner),
            StringCombiner,
        )
        .to_token(token_type)
}

trait StringToTokenParser {
    fn to_token(
        self,
        token_type: TokenType,
    ) -> impl Parser<StringView, Output = Token, Error = ParserError>;
}

impl<P> StringToTokenParser for P
where
    P: Parser<StringView, Output = String, Error = ParserError>,
{
    fn to_token(
        self,
        token_type: TokenType,
    ) -> impl Parser<StringView, Output = Token, Error = ParserError> {
        self.map(move |text| Token::new(token_type.get_index(), text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::create_string_tokenizer;

    fn parse_token(input: &str) -> Token {
        let mut input = create_string_tokenizer(input.to_owned());
        any_token().parse(&mut input).ok().unwrap()
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
