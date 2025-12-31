use rusty_common::Positioned;
use rusty_pc::*;

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{AsBareName, BareName, ExpressionType, HasExpressionType, ToBareName, TypeQualifier};

const MAX_LENGTH: usize = 40;

/// Defines a name.
///
/// A qualified name is a bare name followed by a built-in type qualifier.
/// Example: `name$`, `age%`.
///
/// Parsing syntax reference
///
/// ```txt
/// <qualifier> ::= "!" | "#" | "$" | "%" | "&"
///
/// <bare-name-with-dots-not-keyword> ::= <bare-name-with-dots> AND NOT keyword
/// <bare-name-with-dots> ::= <letter> | <letter><letters-or-digits-or-dots>
///
/// <bare-name-not-keyword> ::= <bare-name> AND NOT keyword
/// <bare-name> ::= <letter> | <letter><letters-or-digits>
///
/// <letters-or-digits-or-dots> ::= <letter-or-digit-or-dot> | <letter-or-digit-or-dot><letters-or-digits-or-dots>
/// <letter-or-digit-or-dot> ::= <letter> | <digit> | "."
///
/// <letters-or-digits> ::= <letter-or-digit> | <letter-or-digit><letters-or-digits>
/// <letter-or-digit> ::= <letter> | <digit>
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name {
    bare_name: BareName,
    opt_q: Option<TypeQualifier>,
}

impl Name {
    pub fn new(bare_name: BareName, opt_q: Option<TypeQualifier>) -> Self {
        Self { bare_name, opt_q }
    }

    pub fn bare(bare_name: BareName) -> Self {
        Self::new(bare_name, None)
    }

    pub fn qualified(bare_name: BareName, q: TypeQualifier) -> Self {
        Self::new(bare_name, Some(q))
    }

    pub fn qualifier(&self) -> Option<TypeQualifier> {
        self.opt_q.as_ref().copied()
    }

    pub fn is_bare(&self) -> bool {
        self.opt_q.is_none()
    }

    pub fn is_of_type(&self, qualifier: TypeQualifier) -> bool {
        self.opt_q.filter(|q| *q == qualifier).is_some()
    }

    pub fn is_bare_or_of_type(&self, qualifier: TypeQualifier) -> bool {
        self.is_bare() || self.is_of_type(qualifier)
    }

    pub fn try_concat_name(self, right: Self) -> Option<Self> {
        if self.is_bare() {
            Some(Self::new(
                Self::dot_concat(self.bare_name, right.bare_name),
                right.opt_q,
            ))
        } else {
            None
        }
    }

    pub fn dot_concat(mut left: BareName, right: BareName) -> BareName {
        left.push('.');
        left.push_str(right.as_ref());
        left
    }

    pub fn demand_bare(self) -> BareName {
        if !self.is_bare() {
            panic!("{:?} was not bare", self)
        }
        self.bare_name
    }

    pub fn demand_qualified(self) -> Name {
        if self.is_bare() {
            panic!("{:?} was not qualified", self)
        }

        self
    }
}

impl AsBareName for Name {
    fn as_bare_name(&self) -> &BareName {
        &self.bare_name
    }
}

impl ToBareName for Name {
    fn to_bare_name(self) -> BareName {
        self.bare_name
    }
}

// TODO #[cfg(test)]
impl From<&str> for Name {
    fn from(s: &str) -> Self {
        let mut buf = s.to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Self::qualified(buf.into(), qualifier),
            _ => {
                buf.push(last_ch);
                Self::bare(buf.into())
            }
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bare_name.fmt(f).and_then(|res| match self.opt_q {
            Some(q) => q.fmt(f),
            _ => Ok(res),
        })
    }
}

impl HasExpressionType for Name {
    fn expression_type(&self) -> ExpressionType {
        match &self.opt_q {
            Some(qualifier) => ExpressionType::BuiltIn(*qualifier),
            None => ExpressionType::Unresolved,
        }
    }
}

/// A [Name] with position information.
pub type NamePos = Positioned<Name>;

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Name::from("A"), Name::bare("A".into()));
        assert_eq!(
            Name::from("Pos%"),
            Name::qualified(
                BareName::new("Pos".to_string()),
                TypeQualifier::PercentInteger
            )
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Name::from("Foo").to_string(), "Foo");
        assert_eq!(Name::from("age%").to_string(), "age%");
    }
}

/// Parses an unqualified name without dots.
///
/// Dots or type qualifier result in a fatal error.
///
/// Errors if it exceeds the maximum length of identifiers.
///
/// Use case: user defined type elements or types.
pub fn bare_name_without_dots() -> impl Parser<RcStringView, Output = BareName, Error = ParseError>
{
    ensure_no_trailing_dot_or_qualifier(identifier()).map(|token| BareName::new(token.to_str()))
}

/// Parses an identifier token.
/// Errors if it exceeds the maximum length of identifiers.
pub fn identifier() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of(TokenType::Identifier).flat_map(ensure_token_length)
}

/// Parses a type qualifier character.
fn type_qualifier_unchecked() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
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
    if token.as_str().chars().count() > MAX_LENGTH {
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
pub fn bare_name_with_dots() -> impl Parser<RcStringView, Output = BareName, Error = ParseError> {
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
pub fn identifier_with_dots() -> impl Parser<RcStringView, Output = TokenList, Error = ParseError> {
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

fn identifier_or_keyword() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token()
        .filter(|token| TokenType::Identifier.matches(token) || TokenType::Keyword.matches(token))
}

fn identifier_or_keyword_or_dot() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
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
        .map(|token| token.as_str().chars().count())
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
pub fn name_with_dots() -> impl Parser<RcStringView, Output = Name, Error = ParseError> {
    name_with_dots_as_tokens().map(Name::from)
}

pub fn name_with_dots_as_tokens()
-> impl Parser<RcStringView, Output = NameAsTokens, Error = ParseError> {
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
pub fn type_qualifier() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    ensure_no_trailing_dot_or_qualifier(type_qualifier_unchecked())
}

fn ensure_no_trailing_dot_or_qualifier<P>(
    parser: impl Parser<RcStringView, Output = P, Error = ParseError>,
) -> impl Parser<RcStringView, Output = P, Error = ParseError> {
    ensure_no_trailing_qualifier(ensure_no_trailing_dot(parser))
}

/// Returns the result of the given parser,
/// but it gives an error if it is followed by a dot.
fn ensure_no_trailing_dot<P>(
    parser: impl Parser<RcStringView, Output = P, Error = ParseError>,
) -> impl Parser<RcStringView, Output = P, Error = ParseError> {
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
    parser: impl Parser<RcStringView, Output = P, Error = ParseError>,
) -> impl Parser<RcStringView, Output = P, Error = ParseError> {
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
        Self::new(
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

pub fn token_list_to_bare_name(tokens: TokenList) -> BareName {
    BareName::new(token_list_to_string(tokens))
}

#[cfg(test)]
mod parse_tests {
    use rusty_pc::Parser;

    use super::*;
    use crate::parametric_test;
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
            assert_eq!(token.as_str(), "Hello");
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
