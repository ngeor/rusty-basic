use rusty_common::Positioned;
use rusty_pc::*;

use crate::core::{VarNameCtx, user_defined_type, var_name};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{any_symbol_of, any_token_of};
use crate::{
    BareNamePos, BuiltInStyle, ExpressionType, HasExpressionType, Keyword, ParserError, TypeQualifier, TypedName, VarType
};

pub type Parameter = TypedName<ParamType>;
pub type ParameterPos = Positioned<Parameter>;
pub type Parameters = Vec<ParameterPos>;

// same as dim minus the "x as string * 5" and the array dimensions
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum ParamType {
    #[default]
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNamePos),
    Array(Box<Self>),
}

impl VarType for ParamType {
    fn new_built_in_compact(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
    }
    fn new_built_in_extended(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Extended)
    }

    fn new_user_defined(bare_name_pos: BareNamePos) -> Self {
        Self::UserDefined(bare_name_pos)
    }

    fn as_user_defined_recursively(&self) -> Option<&BareNamePos> {
        match self {
            Self::UserDefined(n) => Some(n),
            Self::Array(e) => e.as_user_defined_recursively(),
            _ => None,
        }
    }

    fn to_qualifier_recursively(&self) -> Option<TypeQualifier> {
        match self {
            Self::BuiltIn(q, _) => Some(*q),
            Self::Array(e) => e.to_qualifier_recursively(),
            _ => None,
        }
    }

    fn is_extended(&self) -> bool {
        match self {
            Self::BuiltIn(_, BuiltInStyle::Extended) | Self::UserDefined(_) => true,
            Self::Array(element_type) => element_type.is_extended(),
            _ => false,
        }
    }
}

impl HasExpressionType for ParamType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier, _) => ExpressionType::BuiltIn(*qualifier),
            Self::UserDefined(Positioned { element, .. }) => {
                ExpressionType::UserDefined(element.clone())
            }
            Self::Array(boxed_element_type) => {
                ExpressionType::Array(Box::new(boxed_element_type.expression_type()))
            }
            _ => ExpressionType::Unresolved,
        }
    }
}

/// Parses a Param name. Possible options:
/// A
/// A%
/// A.B AS INTEGER       // no qualifier, yes dots
/// A AS UserDefinedType // not dots no qualifiers
/// A() empty array
/// A.B() as INTEGER
pub fn parameter_pos_p() -> impl Parser<StringView, Output = ParameterPos, Error = ParserError> {
    parameter_p().with_pos()
}

fn parameter_p() -> impl Parser<StringView, Output = Parameter, Error = ParserError> {
    var_name(array_indicator, extended_type)
}

fn array_indicator() -> impl Parser<StringView, Output = Option<(Token, Token)>, Error = ParserError>
{
    // TODO support ignoring token to avoid allocation
    seq2(any_symbol_of!('('), any_symbol_of!(')'), |l, r| (l, r)).to_option()
}

fn extended_type() -> impl Parser<StringView, VarNameCtx, Output = ParamType, Error = ParserError> {
    IifCtxParser::new(
        // allow user defined
        built_in_extended_type()
            .or(user_defined_type())
            .with_expected_message("INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier"),
        // do not allow user defined
        built_in_extended_type(),
    )
    .map_ctx(|ctx: &VarNameCtx| ctx.1)
}

fn built_in_extended_type() -> impl Parser<StringView, Output = ParamType, Error = ParserError> {
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
