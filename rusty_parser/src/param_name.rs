use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use crate::var_name::var_name;

/// Parses a Param name. Possible options:
/// A
/// A%
/// A.B AS INTEGER       // no qualifier, yes dots
/// A AS UserDefinedType // not dots no qualifiers
/// A() empty array
/// A.B() as INTEGER
pub fn parameter_pos_p() -> impl Parser<RcStringView, Output = ParameterPos> {
    parameter_p().with_pos()
}

fn parameter_p() -> impl Parser<RcStringView, Output = Parameter> {
    var_name(array_indicator(), built_in_extended_type)
}

fn array_indicator() -> impl Parser<RcStringView, Output = Option<(Token, Token)>> {
    Seq2::new(
        any_token_of(TokenType::LParen),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
    .to_option()
}

fn built_in_extended_type() -> impl Parser<RcStringView, Output = ParamType> {
    // TODO make a keyword_map that doesn't require Clone
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
