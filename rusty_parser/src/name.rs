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
pub fn bare_name_without_dots() -> impl Parser<RcStringView, Output = BareName> {
    ensure_no_trailing_dot_or_qualifier(identifier()).map(BareName::from)
}

/// Parses an identifier token.
/// Errors if it exceeds the maximum length of identifiers.
pub fn identifier() -> impl Parser<RcStringView, Output = Token> {
    any_token_of(TokenType::Identifier).flat_map(ensure_token_length)
}

/// Parses a type qualifier character.
fn type_qualifier_unchecked() -> impl Parser<RcStringView, Output = Token> {
    any_token().filter(is_type_qualifier)
}

fn is_type_qualifier(token: &Token) -> bool {
    TokenType::ExclamationMark.matches(token)
        || TokenType::Pound.matches(token)
        || TokenType::DollarSign.matches(token)
        || TokenType::Percent.matches(token)
        || TokenType::Ampersand.matches(token)
}

fn ensure_token_length(
    input: RcStringView,
    token: Token,
) -> ParseResult<RcStringView, Token, ParseError> {
    if token.text.chars().count() > MAX_LENGTH {
        Err((true, input, ParseError::IdentifierTooLong))
    } else {
        Ok((input, token))
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
pub fn bare_name_with_dots() -> impl Parser<RcStringView, Output = BareName> {
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
pub fn identifier_with_dots() -> impl Parser<RcStringView, Output = TokenList> {
    OrParser::new(vec![
        // to allow keywords, there must be at least one dot
        Box::new(
            seq2(
                identifier_or_keyword().and(dot(), |left, right| vec![left, right]),
                identifier_or_keyword_or_dot().zero_or_more(),
                |mut left_list, mut right_list| {
                    left_list.append(&mut right_list);
                    left_list
                },
            )
            .flat_map(ensure_token_list_length),
        ),
        // otherwise just one identifier (max_length already checked)
        Box::new(identifier().map(|token| vec![token])),
    ])
}

fn identifier_or_keyword() -> impl Parser<RcStringView, Output = Token> {
    any_token()
        .filter(|token| TokenType::Identifier.matches(token) || TokenType::Keyword.matches(token))
}

fn identifier_or_keyword_or_dot() -> impl Parser<RcStringView, Output = Token> {
    any_token().filter(|token| {
        TokenType::Identifier.matches(token)
            || TokenType::Dot.matches(token)
            || TokenType::Keyword.matches(token)
    })
}

// TODO add test: max length of 40 characters applies both to parts and the full string
fn ensure_token_list_length(
    input: RcStringView,
    tokens: TokenList,
) -> ParseResult<RcStringView, TokenList, ParseError> {
    if tokens
        .iter()
        .map(|token| token.text.chars().count())
        .sum::<usize>()
        > MAX_LENGTH
    {
        Err((true, input, ParseError::IdentifierTooLong))
    } else {
        Ok((input, tokens))
    }
}

/// Parses a name that might be qualified and might have dots.
///
/// If a type qualifier exists, it cannot be followed by a dot or a second type qualifier.
///
/// It can also be a keyword followed by the dollar sign (e.g. `END$` is a valid result).
pub fn name_with_dots() -> impl Parser<RcStringView, Output = Name> {
    name_with_dots_as_tokens().map(Name::from)
}

pub fn name_with_dots_as_tokens() -> impl Parser<RcStringView, Output = NameAsTokens> {
    OrParser::new(vec![
        Box::new(identifier_with_dots().and_opt_tuple(type_qualifier())),
        Box::new(
            ensure_no_trailing_dot_or_qualifier(any_keyword_with_dollar_sign())
                .map(|(keyword_token, dollar_token)| (vec![keyword_token], Some(dollar_token))),
        ),
    ])
}

/// Parses a type qualifier character.
/// Fails if the qualifier is followed by a dot or an additional qualifier.
pub fn type_qualifier() -> impl Parser<RcStringView, Output = Token> {
    ensure_no_trailing_dot_or_qualifier(type_qualifier_unchecked())
}

fn ensure_no_trailing_dot_or_qualifier<P>(
    parser: impl Parser<RcStringView, Output = P>,
) -> impl Parser<RcStringView, Output = P> {
    ensure_no_trailing_qualifier(ensure_no_trailing_dot(parser))
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a dot.
fn ensure_no_trailing_dot<P>(
    parser: impl Parser<RcStringView, Output = P>,
) -> impl Parser<RcStringView, Output = P> {
    parser.and_opt_keep_left(peek_token().flat_map_negate_none(|input, token| {
        // TODO a friendlier flat_map that does not alter the input (and does not need it either)
        if TokenType::Dot.matches(&token) {
            Err((true, input, ParseError::IdentifierCannotIncludePeriod))
        } else {
            Ok((input, ()))
        }
    }))
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a type qualifier character.
fn ensure_no_trailing_qualifier<P>(
    parser: impl Parser<RcStringView, Output = P>,
) -> impl Parser<RcStringView, Output = P> {
    parser.and_opt_keep_left(peek_token().flat_map_negate_none(|input, token| {
        if is_type_qualifier(&token) {
            Err((
                true,
                input,
                ParseError::syntax_error("Identifier cannot end with %, &, !, #, or $"),
            ))
        } else {
            Ok((input, ()))
        }
    }))
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
    use crate::pc::Parser;
    use crate::pc_specific::create_string_tokenizer;

    fn assert_fully_parsed<T, E>(result: &ParseResult<RcStringView, T, E>, input: &str) {
        match result {
            Ok((tokenizer, _)) => {
                assert!(
                    tokenizer.is_eof(),
                    "Should have parsed {} completely",
                    input
                );
            }
            _ => {
                panic!("Should have succeeded for {}", input)
            }
        }
    }

    fn assert_result<T, E>(input: &str, result: ParseResult<RcStringView, T, E>) -> T {
        match result {
            Ok((_, x)) => x,
            _ => panic!("Should have succeeded for {}", input),
        }
    }

    fn assert_err<T, E>(input: &str, result: ParseResult<RcStringView, T, E>, expected_err: E)
    where
        E: std::fmt::Debug + PartialEq,
    {
        match result {
            Err((_, _, err)) => {
                assert_eq!(err, expected_err);
            }
            _ => {
                panic!("Should have failed for {}", input)
            }
        }
    }

    mod bare_name_without_dots {
        use super::*;

        fn parse_something_completely(input: &str) -> BareName {
            let tokenizer = create_string_tokenizer(input.to_owned());
            let result = bare_name_without_dots().parse(tokenizer);
            assert_fully_parsed(&result, input);
            assert_result(input, result)
        }

        #[test]
        fn can_parse_identifier() {
            assert_eq!(parse_something_completely("Hello"), BareName::from("Hello"))
        }

        #[test]
        fn cannot_have_dots() {
            for input in ["Hell.o", "Hello."] {
                let result =
                    bare_name_without_dots().parse(create_string_tokenizer(String::from(input)));
                assert_err(input, result, ParseError::IdentifierCannotIncludePeriod);
            }
        }

        #[test]
        fn cannot_have_trailing_qualifier() {
            for input in ["Hello!", "Hello#", "Hello$", "Hello%", "Hello&"] {
                let result =
                    bare_name_with_dots().parse(create_string_tokenizer(String::from(input)));
                assert_err(
                    input,
                    result,
                    ParseError::syntax_error("Identifier cannot end with %, &, !, #, or $"),
                );
            }
        }

        #[test]
        fn cannot_exceed_max_length() {
            let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO".to_owned();
            assert_eq!(input.len(), 41);
            let result = bare_name_with_dots().parse(create_string_tokenizer(input.clone()));
            assert_err(&input, result, ParseError::IdentifierTooLong);
        }
    }

    mod identifier {
        use super::*;

        fn parse_something_completely(input: &str) -> Token {
            let tokenizer = create_string_tokenizer(input.to_owned());
            let result = identifier().parse(tokenizer);
            assert_fully_parsed(&result, input);
            assert_result(input, result)
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
            let result = identifier().parse(create_string_tokenizer(input.clone()));
            assert_err(&input, result, ParseError::IdentifierTooLong);
        }
    }

    mod bare_name_with_dots {
        use super::*;

        fn parse_something_completely(input: &str) -> BareName {
            let tokenizer = create_string_tokenizer(input.to_owned());
            let result = bare_name_with_dots().parse(tokenizer);
            assert_fully_parsed(&result, input);
            assert_result(input, result)
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
            let tokenizer = create_string_tokenizer(input.to_owned());
            let result = name_with_dots().parse(tokenizer);
            assert_fully_parsed(&result, input);
            assert_result(input, result)
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
