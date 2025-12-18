use crate::dim_name::type_definition::built_in_extended_type;
use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use crate::var_name::var_name;

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
pub fn dim_var_pos_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = DimVarPos> {
    dim_or_redim(array_dimensions::array_dimensions_p().or_default())
}

pub fn redim_var_pos_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = DimVarPos> {
    dim_or_redim(
        array_dimensions::array_dimensions_p().or_syntax_error("Expected: array dimensions"),
    )
}

fn dim_or_redim<I: Tokenizer + 'static>(
    array_dimensions_parser: impl Parser<I, Output = ArrayDimensions>,
) -> impl Parser<I, Output = DimVarPos> {
    var_name(array_dimensions_parser, built_in_extended_type).with_pos()
}

mod array_dimensions {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::*;

    pub fn array_dimensions_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = ArrayDimensions>
    {
        in_parenthesis(csv_non_opt(
            array_dimension_p(),
            "Expected: array dimension",
        ))
    }

    // expr (e.g. 10)
    // expr ws+ TO ws+ expr (e.g. 1 TO 10)
    // paren_expr ws* TO ws* paren_expr
    fn array_dimension_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = ArrayDimension> {
        opt_second_expression_after_keyword(expression::expression_pos_p(), Keyword::To).map(
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
    use crate::expression::expression_pos_p;
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::*;

    pub fn built_in_extended_type<I: Tokenizer + 'static>() -> impl Parser<I, Output = DimType> {
        OrParser::new(vec![
            Box::new(built_in_numeric_type()),
            Box::new(built_in_string()),
        ])
        .with_expected_message("Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING")
    }

    fn built_in_numeric_type<I: Tokenizer + 'static>() -> impl Parser<I, Output = DimType> {
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

    fn built_in_string<I: Tokenizer + 'static>() -> impl Parser<I, Output = DimType> {
        keyword(Keyword::String)
            .and_opt(
                star().then_demand(
                    expression_pos_p().or_syntax_error("Expected: string length after *"),
                ),
            )
            .map(|(_, opt_len)| match opt_len {
                Some(len) => DimType::FixedLengthString(len, 0),
                _ => DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
            })
    }
}
