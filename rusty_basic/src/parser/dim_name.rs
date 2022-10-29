use crate::parser::dim_name::type_definition::built_in_extended_type;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use crate::parser::var_name::var_name;

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
pub fn dim_name_node_p() -> impl Parser<Output = DimNameNode> {
    dim_or_redim(array_dimensions::array_dimensions_p().allow_default())
}

pub fn redim_name_node_p() -> impl Parser<Output = DimNameNode> {
    dim_or_redim(
        array_dimensions::array_dimensions_p().or_syntax_error("Expected: array dimensions"),
    )
}

fn dim_or_redim(
    array_dimensions_parser: impl Parser<Output = ArrayDimensions> + NonOptParser,
) -> impl Parser<Output = DimNameNode> {
    var_name(array_dimensions_parser, built_in_extended_type).with_pos()
}

mod array_dimensions {
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn array_dimensions_p() -> impl Parser<Output = ArrayDimensions> {
        in_parenthesis(csv_non_opt(
            array_dimension_p(),
            "Expected: array dimension",
        ))
    }

    // expr (e.g. 10)
    // expr ws+ TO ws+ expr (e.g. 1 TO 10)
    // paren_expr ws* TO ws* paren_expr
    fn array_dimension_p() -> impl Parser<Output = ArrayDimension> {
        opt_second_expression_after_keyword(expression::expression_node_p(), Keyword::To).map(
            |(l, opt_r)| match opt_r {
                Some(r) => ArrayDimension {
                    lbound: Some(l),
                    ubound: r,
                },
                None => ArrayDimension {
                    lbound: None,
                    ubound: l,
                },
            },
        )
    }
}

mod type_definition {
    use crate::parser::expression::expression_node_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;
    use rusty_common::*;

    pub fn built_in_extended_type() -> impl Parser<Output = DimType> {
        Alt2::new(built_in_numeric_type(), built_in_string()).map_incomplete_err(QError::expected(
            "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING",
        ))
    }

    fn built_in_numeric_type() -> impl Parser<Output = DimType> {
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

    fn built_in_string() -> impl Parser<Output = DimType> {
        keyword(Keyword::String)
            .and_opt(star().then_demand(
                expression_node_p().or_syntax_error("Expected: string length after *"),
            ))
            .map(|(_, opt_len)| match opt_len {
                Some(len) => DimType::FixedLengthString(len, 0),
                _ => DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
            })
    }
}
