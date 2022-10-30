use crate::pc::*;
use crate::pc_specific::*;
use crate::{BareName, Name, TypeQualifier};
use rusty_common::*;

const MAX_LENGTH: usize = 40;

/// Parses an unqualified name without dots.
///
/// Dots or type qualifier result in a fatal error.
///
/// Errors if it exceeds the maximum length of identifiers.
///
/// Use case: user defined type elements or types.
pub fn bare_name_without_dots() -> impl Parser<Output = BareName> {
    ensure_no_trailing_dot_or_qualifier(identifier()).map(BareName::from)
}

/// Parses an identifier token.
/// Errors if it exceeds the maximum length of identifiers.
pub fn identifier() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Identifier).and_then(ensure_token_length)
}

/// Parses a type qualifier character.
fn type_qualifier_unchecked() -> impl Parser<Output = Token> {
    any_token().filter(is_type_qualifier)
}

fn is_type_qualifier(token: &Token) -> bool {
    TokenType::ExclamationMark.matches(token)
        || TokenType::Pound.matches(token)
        || TokenType::DollarSign.matches(token)
        || TokenType::Percent.matches(token)
        || TokenType::Ampersand.matches(token)
}

fn ensure_token_length(token: Token) -> Result<Token, QError> {
    if token.text.chars().count() > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        Ok(token)
    }
}

/// Parses an unqualified name that might have dots.
///
/// A trailing type qualifier is a fatal error.
///
/// Errors if it exceeds the maximum length of identifiers.
///
/// Trailing dots are allowed, as well as consecutive dots
/// within the resulting name. Keywords can also be part
/// of the name (e.g. `VIEW.PRINT` is a valid result).
///
/// Usages: SUB name and labels.
pub fn bare_name_with_dots() -> impl Parser<Output = BareName> {
    ensure_no_trailing_qualifier(identifier_with_dots()).map(token_list_to_bare_name)
}

/// Parses a sequence of name-like tokens that might include dots.
///
/// Errors if it exceeds the maximum length of identifiers.
///
/// Trailing dots are allowed, as well as consecutive dots
/// within the resulting name. Keywords can also be part
/// of the name (e.g. `VIEW.PRINT` is a valid result).
///
/// Usage: label declaration (but also used internally in the module).
pub fn identifier_with_dots() -> impl Parser<Output = TokenList> {
    Alt2::new(
        // to allow keywords, there must be at least one dot
        seq2(
            identifier_or_keyword()
                .and(dot())
                .map(|(left, right)| vec![left, right]),
            identifier_or_keyword_or_dot().zero_or_more(),
            |mut left_list, mut right_list| {
                left_list.append(&mut right_list);
                left_list
            },
        )
        .and_then(ensure_token_list_length),
        // otherwise just one identifier (max_length already checked)
        identifier().map(|token| vec![token]),
    )
}

fn identifier_or_keyword() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| TokenType::Identifier.matches(token) || TokenType::Keyword.matches(token))
}

fn identifier_or_keyword_or_dot() -> impl Parser<Output = Token> {
    any_token().filter(|token| {
        TokenType::Identifier.matches(token)
            || TokenType::Dot.matches(token)
            || TokenType::Keyword.matches(token)
    })
}

// TODO add test: max length of 40 characters applies both to parts and the full string
fn ensure_token_list_length(tokens: TokenList) -> Result<TokenList, QError> {
    if tokens
        .iter()
        .map(|token| token.text.chars().count())
        .sum::<usize>()
        > MAX_LENGTH
    {
        Err(QError::IdentifierTooLong)
    } else {
        Ok(tokens)
    }
}

/// Parses a name that might be qualified and might have dots.
///
/// If a type qualifier exists, it cannot be followed by a dot or a second type qualifier.
///
/// It can also be a keyword followed by the dollar sign (e.g. `END$` is a valid result).
pub fn name_with_dots() -> impl Parser<Output = Name> {
    name_with_dots_as_tokens().map(Name::from)
}

pub fn name_with_dots_as_tokens() -> impl Parser<Output = NameAsTokens> {
    Alt2::new(
        identifier_with_dots().and_opt(type_qualifier()),
        ensure_no_trailing_dot_or_qualifier(any_keyword_with_dollar_sign())
            .map(|(keyword_token, dollar_token)| (vec![keyword_token], Some(dollar_token))),
    )
}

/// Parses a type qualifier character.
/// Fails if the qualifier is followed by a dot or an additional qualifier.
pub fn type_qualifier() -> impl Parser<Output = Token> {
    ensure_no_trailing_dot_or_qualifier(type_qualifier_unchecked())
}

fn ensure_no_trailing_dot_or_qualifier<P>(
    parser: impl Parser<Output = P>,
) -> impl Parser<Output = P> {
    ensure_no_trailing_qualifier(ensure_no_trailing_dot(parser))
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a dot.
fn ensure_no_trailing_dot<P>(parser: impl Parser<Output = P>) -> impl Parser<Output = P> {
    seq2(
        parser,
        dot()
            .peek()
            .negate()
            .or_fail(QError::IdentifierCannotIncludePeriod),
        |l, _| l,
    )
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a type qualifier character.
fn ensure_no_trailing_qualifier<P>(parser: impl Parser<Output = P>) -> impl Parser<Output = P> {
    seq2(
        parser,
        type_qualifier_unchecked()
            .peek()
            .negate()
            .or_syntax_error("Identifier cannot end with %, &, !, #, or $"),
        |l, _| l,
    )
}

pub type NameAsTokens = (TokenList, Option<Token>);

impl From<NameAsTokens> for Name {
    fn from(name_as_tokens: NameAsTokens) -> Self {
        let (name_tokens, opt_q) = name_as_tokens;
        Name::new(
            token_list_to_bare_name(name_tokens),
            opt_q.as_ref().map(token_to_type_qualifier),
        )
    }
}

pub fn token_to_type_qualifier(token: &Token) -> TypeQualifier {
    if TokenType::ExclamationMark.matches(token) {
        TypeQualifier::BangSingle
    } else if TokenType::Pound.matches(token) {
        TypeQualifier::HashDouble
    } else if TokenType::DollarSign.matches(token) {
        TypeQualifier::DollarString
    } else if TokenType::Percent.matches(token) {
        TypeQualifier::PercentInteger
    } else if TokenType::Ampersand.matches(token) {
        TypeQualifier::AmpersandLong
    } else {
        panic!("Unsupported token")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parametric_test;
    use crate::test_utils::*;

    mod bare_name_without_dots {
        use super::*;

        #[test]
        fn can_parse_identifier() {
            assert_eq!(
                parse_something_completely("Hello", bare_name_without_dots()),
                BareName::from("Hello")
            )
        }

        #[test]
        fn cannot_have_dots() {
            for input in &["Hell.o", "Hello."] {
                assert_eq!(
                    parse_something(input, bare_name_without_dots()).expect_err("Should fail"),
                    QError::IdentifierCannotIncludePeriod
                );
            }
        }

        #[test]
        fn cannot_have_trailing_qualifier() {
            for input in &["Hello!", "Hello#", "Hello$", "Hello%", "Hello&"] {
                assert_eq!(
                    parse_something(input, bare_name_without_dots()).expect_err("Should fail"),
                    QError::syntax_error("Identifier cannot end with %, &, !, #, or $")
                );
            }
        }

        #[test]
        fn cannot_exceed_max_length() {
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO";
            assert_eq!(input.len(), 41);
            assert_eq!(
                parse_something(input, bare_name_without_dots()).expect_err("Should fail"),
                QError::IdentifierTooLong
            );
        }
    }

    mod identifier {
        use super::*;

        #[test]
        fn can_parse_identifier() {
            let token = parse_something_completely("Hello", identifier());
            assert!(TokenType::Identifier.matches(&token));
            assert_eq!(token.text, "Hello");
        }

        #[test]
        fn cannot_exceed_max_length() {
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO";
            assert_eq!(input.len(), 41);
            assert_eq!(
                parse_something(input, identifier()).expect_err("Should fail"),
                QError::IdentifierTooLong
            );
        }
    }

    mod bare_name_with_dots {
        use super::*;

        fn happy_flow(input: &str) {
            let parser = bare_name_with_dots();
            let result = parse_something_completely(input, parser);
            assert_eq!(result, BareName::from(input));
        }

        parametric_test!(
            happy_flow,
            [
                can_parse_abc,
                "abc",
                can_parse_abc1,
                "abc1",
                can_parse_abc_dot_def,
                "abc.def",
                can_parse_abc_dot_dot,
                "abc..",
                can_parse_def_dot_string,
                "def.string",
                can_parse_print_dot_view_dot,
                "print.view.",
            ]
        );
    }

    mod name_with_dots {
        use super::*;

        fn happy_flow(input: &str) {
            let parser = name_with_dots();
            let result = parse_something_completely(input, parser);
            assert_eq!(result, Name::from(input));
        }

        parametric_test!(
            happy_flow,
            [
                can_parse_abc,
                "abc",
                can_parse_abc1,
                "abc1",
                can_parse_abc_dot_def,
                "abc.def",
                can_parse_abc_dot_dot,
                "abc..",
                can_parse_def_dot_string,
                "def.string",
                can_parse_print_dot_view_dot,
                "print.view.",
                can_parse_end_dollar_sign,
                "end$",
                can_parse_hello_single,
                "hello!",
                can_parse_hello_double,
                "hello#",
                can_parse_hello_string,
                "hello$",
                can_parse_hello_integer,
                "hello%",
                can_parse_hello_long,
                "hello&",
            ]
        );
    }
}
