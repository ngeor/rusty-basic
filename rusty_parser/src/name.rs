use crate::pc::*;
use crate::pc_specific::*;
use crate::{BareName, Name, ParseError, TypeQualifier};

const MAX_LENGTH: usize = 40;

/// Parses an unqualified name without dots.
///
/// Dots or type qualifier result in a fatal error.
///
/// Errors if it exceeds the maximum length of identifiers.
///
/// Use case: user defined type elements or types.
pub fn bare_name_without_dots<I: Tokenizer + 'static>() -> impl Parser<I, Output = BareName> {
    ensure_no_trailing_dot_or_qualifier(identifier()).map(BareName::from)
}

/// Parses an identifier token.
/// Errors if it exceeds the maximum length of identifiers.
pub fn identifier<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Identifier).and_then(ensure_token_length)
}

/// Parses a type qualifier character.
fn type_qualifier_unchecked<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token().filter(is_type_qualifier)
}

fn is_type_qualifier(token: &Token) -> bool {
    TokenType::ExclamationMark.matches(token)
        || TokenType::Pound.matches(token)
        || TokenType::DollarSign.matches(token)
        || TokenType::Percent.matches(token)
        || TokenType::Ampersand.matches(token)
}

fn ensure_token_length(token: Token) -> Result<Token, ParseError> {
    if token.text.chars().count() > MAX_LENGTH {
        Err(ParseError::IdentifierTooLong)
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
pub fn bare_name_with_dots<I: Tokenizer + 'static>() -> impl Parser<I, Output = BareName> {
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
pub fn identifier_with_dots<I: Tokenizer + 'static>() -> impl Parser<I, Output = TokenList> {
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

fn identifier_or_keyword<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token()
        .filter(|token| TokenType::Identifier.matches(token) || TokenType::Keyword.matches(token))
}

fn identifier_or_keyword_or_dot<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token().filter(|token| {
        TokenType::Identifier.matches(token)
            || TokenType::Dot.matches(token)
            || TokenType::Keyword.matches(token)
    })
}

// TODO add test: max length of 40 characters applies both to parts and the full string
fn ensure_token_list_length(tokens: TokenList) -> Result<TokenList, ParseError> {
    if tokens
        .iter()
        .map(|token| token.text.chars().count())
        .sum::<usize>()
        > MAX_LENGTH
    {
        Err(ParseError::IdentifierTooLong)
    } else {
        Ok(tokens)
    }
}

/// Parses a name that might be qualified and might have dots.
///
/// If a type qualifier exists, it cannot be followed by a dot or a second type qualifier.
///
/// It can also be a keyword followed by the dollar sign (e.g. `END$` is a valid result).
pub fn name_with_dots<I: Tokenizer + 'static>() -> impl Parser<I, Output = Name> {
    name_with_dots_as_tokens().map(Name::from)
}

pub fn name_with_dots_as_tokens<I: Tokenizer + 'static>() -> impl Parser<I, Output = NameAsTokens> {
    Alt2::new(
        identifier_with_dots().and_opt(type_qualifier()),
        ensure_no_trailing_dot_or_qualifier(any_keyword_with_dollar_sign())
            .map(|(keyword_token, dollar_token)| (vec![keyword_token], Some(dollar_token))),
    )
}

/// Parses a type qualifier character.
/// Fails if the qualifier is followed by a dot or an additional qualifier.
pub fn type_qualifier<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    ensure_no_trailing_dot_or_qualifier(type_qualifier_unchecked())
}

fn ensure_no_trailing_dot_or_qualifier<I: Tokenizer + 'static, P>(
    parser: impl Parser<I, Output = P>,
) -> impl Parser<I, Output = P> {
    ensure_no_trailing_qualifier(ensure_no_trailing_dot(parser))
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a dot.
fn ensure_no_trailing_dot<I: Tokenizer + 'static, P>(
    parser: impl Parser<I, Output = P>,
) -> impl Parser<I, Output = P> {
    seq2(
        parser,
        dot()
            .peek()
            .negate()
            .or_fail(ParseError::IdentifierCannotIncludePeriod),
        |l, _| l,
    )
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a type qualifier character.
fn ensure_no_trailing_qualifier<I: Tokenizer + 'static, P>(
    parser: impl Parser<I, Output = P>,
) -> impl Parser<I, Output = P> {
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
    use crate::pc::{Parser, Tokenizer};
    use crate::pc_specific::create_string_tokenizer;

    mod bare_name_without_dots {
        use super::*;

        fn parse_something_completely(input: &str) -> BareName {
            let mut tokenizer = create_string_tokenizer(input.to_owned());
            let result = bare_name_without_dots()
                .parse(&mut tokenizer)
                .unwrap_or_else(|_| panic!("Should have succeeded for {}", input));
            assert!(
                tokenizer.read().expect("Should read EOF token").is_none(),
                "Should have parsed {} completely",
                input
            );
            result
        }

        #[test]
        fn can_parse_identifier() {
            assert_eq!(parse_something_completely("Hello"), BareName::from("Hello"))
        }

        #[test]
        fn cannot_have_dots() {
            for input in ["Hell.o", "Hello."] {
                let input: String = String::from(input);
                let result = bare_name_with_dots().parse(&mut create_string_tokenizer(input));
                assert_eq!(
                    result.expect_err("Should fail"),
                    ParseError::IdentifierCannotIncludePeriod
                );
            }
        }

        #[test]
        fn cannot_have_trailing_qualifier() {
            for input in ["Hello!", "Hello#", "Hello$", "Hello%", "Hello&"] {
                let input: String = String::from(input);
                let result = bare_name_with_dots().parse(&mut create_string_tokenizer(input));
                assert_eq!(
                    result.expect_err("Should fail"),
                    ParseError::syntax_error("Identifier cannot end with %, &, !, #, or $")
                );
            }
        }

        #[test]
        fn cannot_exceed_max_length() {
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO".to_owned();
            assert_eq!(input.len(), 41);
            let result = bare_name_with_dots().parse(&mut create_string_tokenizer(input));
            assert_eq!(
                result.expect_err("Should fail"),
                ParseError::IdentifierTooLong
            );
        }
    }

    mod identifier {
        use super::*;

        fn parse_something_completely(input: &str) -> Token {
            let mut tokenizer = create_string_tokenizer(input.to_owned());
            let result = identifier()
                .parse(&mut tokenizer)
                .unwrap_or_else(|_| panic!("Should have succeeded for {}", input));
            assert!(
                tokenizer.read().expect("Should read EOF token").is_none(),
                "Should have parsed {} completely",
                input
            );
            result
        }

        #[test]
        fn can_parse_identifier() {
            let token = parse_something_completely("Hello");
            assert!(TokenType::Identifier.matches(&token));
            assert_eq!(token.text, "Hello");
        }

        #[test]
        fn cannot_exceed_max_length() {
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO".to_owned();
            assert_eq!(input.len(), 41);
            let result = identifier().parse(&mut create_string_tokenizer(input));
            assert_eq!(
                result.expect_err("Should fail"),
                ParseError::IdentifierTooLong
            );
        }
    }

    mod bare_name_with_dots {
        use super::*;

        fn parse_something_completely(input: &str) -> BareName {
            let mut tokenizer = create_string_tokenizer(input.to_owned());
            let result = bare_name_with_dots()
                .parse(&mut tokenizer)
                .unwrap_or_else(|_| panic!("Should have succeeded for {}", input));
            assert!(
                tokenizer.read().expect("Should read EOF token").is_none(),
                "Should have parsed {} completely",
                input
            );
            result
        }

        fn happy_flow(input: &str) {
            let result = parse_something_completely(input);
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

        fn parse_something_completely(input: &str) -> Name {
            let mut tokenizer = create_string_tokenizer(input.to_owned());
            let result = name_with_dots()
                .parse(&mut tokenizer)
                .unwrap_or_else(|_| panic!("Should have succeeded for {}", input));
            assert!(
                tokenizer.read().expect("Should read EOF token").is_none(),
                "Should have parsed {} completely",
                input
            );
            result
        }

        fn happy_flow(input: &str) {
            let result = parse_something_completely(input);
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
