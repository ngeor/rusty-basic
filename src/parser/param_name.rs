use crate::common::*;
use crate::parser::name::{bare_name_without_dots, name_with_dots};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

/// Parses a Param name. Possible options:
/// A
/// A%
/// A.B AS INTEGER       // no qualifier, yes dots
/// A AS UserDefinedType // not dots no qualifiers
/// A() empty array
/// A.B() as INTEGER
pub fn param_name_node_p() -> impl Parser<Output = ParamNameNode> {
    param_name().with_pos()
}

fn param_name() -> impl Parser<Output = ParamName> {
    Seq2::new(name_with_dots(), array_indicator())
        .chain(param_type)
        .map(|(name, array_indicator, param_type)| {
            map_to_param_name(name, array_indicator, param_type)
        })
}

type ArrayIndicator = Option<(Token, Token)>;

fn array_indicator() -> impl Parser<Output = ArrayIndicator> + NonOptParser {
    Seq2::new(
        any_token_of(TokenType::LParen),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
    .allow_none()
}

fn param_type(
    input: (Name, ArrayIndicator),
) -> impl ParserOnce<Output = (Name, ArrayIndicator, ParamType)> {
    let (name, array_indicator) = input;
    let has_dots = name.bare_name().contains('.');
    match_option_p(
        name.qualifier(),
        // qualified name can't have an "AS" clause
        |q| once_p(ParamType::BuiltIn(q, BuiltInStyle::Compact)),
        // bare names might have an "AS" clause
        move || {
            as_clause()
                .then_demand(
                    iif_p(has_dots, built_in_extended_type(), extended_type()).no_incomplete(),
                )
                .to_parser_once()
                .or(once_p(ParamType::Bare))
        },
    )
    .map(|param_type| (name, array_indicator, param_type))
}

fn as_clause() -> impl Parser<Output = (Token, Token, Token)> {
    seq2(
        whitespace().and(keyword(Keyword::As)),
        whitespace().no_incomplete(),
        |(a, b), c| (a, b, c),
    )
}

fn built_in_extended_type() -> impl Parser<Output = ParamType> {
    keyword_map(&[
        (
            Keyword::Single,
            ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Extended),
        ),
        (
            Keyword::Double,
            ParamType::BuiltIn(TypeQualifier::HashDouble, BuiltInStyle::Extended),
        ),
        (
            Keyword::String,
            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
        ),
        (
            Keyword::Integer,
            ParamType::BuiltIn(TypeQualifier::PercentInteger, BuiltInStyle::Extended),
        ),
        (
            Keyword::Long,
            ParamType::BuiltIn(TypeQualifier::AmpersandLong, BuiltInStyle::Extended),
        ),
    ])
}

fn user_defined_type() -> impl Parser<Output = ParamType> {
    bare_name_without_dots()
        .with_pos()
        .map(ParamType::UserDefined)
}

fn extended_type() -> impl Parser<Output = ParamType> {
    built_in_extended_type()
        .or(user_defined_type())
        .map_incomplete_err(QError::expected("Expected: extended type"))
}

fn map_to_param_name(
    name: Name,
    array_indicator: ArrayIndicator,
    param_type: ParamType,
) -> ParamName {
    ParamName::new(
        name.into(),
        match array_indicator {
            Some(_) => ParamType::Array(Box::new(param_type)),
            _ => param_type,
        },
    )
}
