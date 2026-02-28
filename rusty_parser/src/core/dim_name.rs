use rusty_common::*;
use rusty_pc::*;

use crate::core::var_name;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{
    ArrayDimensions, BareName, BuiltInStyle, DimList, DimType, Name, ParserError, ToBareName,
    TypeQualifier, TypedName,
};

pub type DimVar = TypedName<DimType>;
pub type DimVarPos = Positioned<DimVar>;
pub type DimVars = Vec<DimVarPos>;

impl DimVar {
    pub fn new_compact_local<T>(bare_name: T, qualifier: TypeQualifier) -> Self
    where
        BareName: From<T>,
    {
        Self::new(
            BareName::from(bare_name),
            DimType::BuiltIn(qualifier, BuiltInStyle::Compact),
        )
    }

    pub fn into_list(self, pos: Position) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.at_pos(pos)],
        }
    }

    // TODO #[cfg(test)]
    pub fn into_list_rc(self, row: u32, col: u32) -> DimList {
        self.into_list(Position::new(row, col))
    }

    // TODO #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        let qualified_name = Name::from(s).demand_qualified();
        Self::from(qualified_name)
    }
}

impl From<Name> for DimVar {
    fn from(name: Name) -> Self {
        match name.qualifier() {
            Some(qualifier) => Self::new_compact_local(name.to_bare_name(), qualifier),
            _ => Self::new(name.to_bare_name(), DimType::Bare),
        }
    }
}

#[derive(Default)]
// TODO #[deprecated]
pub struct DimNameBuilder {
    pub bare_name: Option<BareName>,
    pub dim_type: Option<DimType>,
}

impl DimNameBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bare_name<T>(mut self, bare_name: T) -> Self
    where
        BareName: From<T>,
    {
        self.bare_name = Some(BareName::from(bare_name));
        self
    }

    pub fn dim_type(mut self, dim_type: DimType) -> Self {
        self.dim_type = Some(dim_type);
        self
    }

    pub fn build(self) -> DimVar {
        DimVar::new(self.bare_name.unwrap(), self.dim_type.unwrap())
    }

    pub fn build_list(self, pos: Position) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.build().at_pos(pos)],
        }
    }

    // TODO #[cfg(test)]
    pub fn build_list_rc(self, row: u32, col: u32) -> DimList {
        self.build_list(Position::new(row, col))
    }
}

/// Parses a declared name. Possible options:
/// A
/// A%
/// A AS INTEGER
/// A AS UserDefinedType
///
/// Arrays:
/// A(10)
/// A$(1 TO 2, 0 TO 10)
/// A(1 TO 5) AS INTEGER
pub fn dim_var_pos_p() -> impl Parser<StringView, Output = DimVarPos, Error = ParserError> {
    dim_or_redim(|| array_dimensions::array_dimensions_p().or_default())
}

pub fn redim_var_pos_p() -> impl Parser<StringView, Output = DimVarPos, Error = ParserError> {
    dim_or_redim(|| array_dimensions::array_dimensions_p().or_expected("array dimensions"))
}

fn dim_or_redim<A, AP>(
    array_dimensions_parser: A,
) -> impl Parser<StringView, Output = DimVarPos, Error = ParserError>
where
    A: Fn() -> AP,
    AP: Parser<StringView, Output = ArrayDimensions, Error = ParserError> + 'static,
{
    var_name(array_dimensions_parser, type_definition::extended_type).with_pos()
}

mod array_dimensions {
    use rusty_pc::*;

    use crate::expr::expr_keyword_opt_expr;
    use crate::input::StringView;
    use crate::pc_specific::*;
    use crate::{ArrayDimension, ArrayDimensions, Keyword, ParserError};

    pub fn array_dimensions_p()
    -> impl Parser<StringView, Output = ArrayDimensions, Error = ParserError> {
        in_parenthesis(csv_non_opt(array_dimension_p(), "array dimension"))
    }

    // expr (e.g. 10)
    // expr ws+ TO ws+ expr (e.g. 1 TO 10)
    // paren_expr ws* TO ws* paren_expr
    fn array_dimension_p() -> impl Parser<StringView, Output = ArrayDimension, Error = ParserError>
    {
        expr_keyword_opt_expr(Keyword::To).map(|(l, opt_r)| match opt_r {
            Some(r) => ArrayDimension {
                lbound: Some(l),
                ubound: r,
            },
            None => ArrayDimension {
                lbound: None,
                ubound: l,
            },
        })
    }
}

mod type_definition {
    use rusty_pc::*;

    use crate::core::{VarNameCtx, user_defined_type};
    use crate::expr::expression_pos_p;
    use crate::input::StringView;
    use crate::pc_specific::*;
    use crate::tokens::star_ws;
    use crate::{BuiltInStyle, DimType, ExpressionPos, Keyword, ParserError, TypeQualifier};

    pub fn extended_type()
    -> impl Parser<StringView, VarNameCtx, Output = DimType, Error = ParserError> {
        IifCtxParser::new(
            // allow user defined
            OrParser::new(vec![
                Box::new(built_in_numeric_type()),
                Box::new(built_in_string()),
                Box::new(user_defined_type()),
            ])
            .with_expected_message("INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier"),
            // do not allow user defined
            OrParser::new(vec![
                Box::new(built_in_numeric_type()),
                Box::new(built_in_string()),
            ])
            .with_expected_message("INTEGER or LONG or SINGLE or DOUBLE or STRING"),
        )
        .map_ctx(|(_, allow_user_defined): &(_, bool)| *allow_user_defined)
    }

    fn built_in_numeric_type() -> impl Parser<StringView, Output = DimType, Error = ParserError> {
        keyword_map(&[
            (
                Keyword::Single,
                DimType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Extended),
            ),
            (
                Keyword::Double,
                DimType::BuiltIn(TypeQualifier::HashDouble, BuiltInStyle::Extended),
            ),
            (
                Keyword::Integer,
                DimType::BuiltIn(TypeQualifier::PercentInteger, BuiltInStyle::Extended),
            ),
            (
                Keyword::Long,
                DimType::BuiltIn(TypeQualifier::AmpersandLong, BuiltInStyle::Extended),
            ),
        ])
    }

    fn built_in_string() -> impl Parser<StringView, Output = DimType, Error = ParserError> {
        keyword(Keyword::String)
            .and_keep_right(star_and_length().to_option())
            .map(|opt_len| match opt_len {
                Some(len) => DimType::FixedLengthString(len, 0),
                _ => DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
            })
    }

    fn star_and_length() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        star_ws().and_keep_right(expression_pos_p().or_expected("string length after *"))
    }
}
