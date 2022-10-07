use crate::common::*;
use crate::parser::name::name_with_dot_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType
//
// Arrays:
// A(10)
// A$(1 TO 2, 0 TO 10)
// A(1 TO 5) AS INTEGER

pub fn dim_name_node_p() -> impl Parser<Output = DimNameNode> {
    Seq3::new(
        name_with_dot_p().with_pos(),
        array_dimensions::array_dimensions_p().allow_default(),
        type_definition::type_definition_extended_p(),
    )
    .and_then(
        |(name_node, array_dimensions, opt_extended_type_definition)| {
            map_name_opt_extended_type_definition(
                name_node,
                array_dimensions,
                opt_extended_type_definition,
            )
        },
    )
}

pub fn redim_name_node_p() -> impl Parser<Output = DimNameNode> {
    Seq3::new(
        name_with_dot_p().with_pos(),
        array_dimensions::array_dimensions_p().or_syntax_error("Expected: array dimensions"),
        type_definition::type_definition_extended_p(),
    )
    .and_then(
        |(name_node, array_dimensions, opt_extended_type_definition)| {
            map_name_opt_extended_type_definition(
                name_node,
                array_dimensions,
                opt_extended_type_definition,
            )
        },
    )
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
        OptSecondExpressionParser::new(expression::expression_node_p(), Keyword::To).map(
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
    use crate::common::*;
    use crate::parser::expression::expression_node_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn type_definition_extended_p() -> impl Parser<Output = Option<DimType>> + NonOptParser {
        // <ws+> AS <ws+> identifier
        seq3(
            whitespace().and(keyword(Keyword::As)),
            whitespace().no_incomplete(),
            extended_type_p().or_syntax_error(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
            ),
            |_, _, identifier| identifier,
        )
        .allow_none()
    }

    fn extended_type_p() -> impl Parser<Output = DimType> {
        Alt3::new(
            user_defined_type(),
            built_in_numeric_type(),
            built_in_string(),
        )
    }

    fn user_defined_type() -> impl Parser<Output = DimType> {
        identifier_with_dots()
            .and_then(|token| {
                if token.text.contains('.') {
                    Err(QError::IdentifierCannotIncludePeriod)
                } else {
                    Ok(BareName::from(token.text))
                }
            })
            .with_pos()
            .map(DimType::UserDefined)
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
        keyword(Keyword::String_)
            .and_opt(star().then_demand(
                expression_node_p().or_syntax_error("Expected: string length after *"),
            ))
            .map(|(_, opt_len)| match opt_len {
                Some(len) => DimType::FixedLengthString(len, 0),
                _ => DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
            })
    }
}

fn map_name_opt_extended_type_definition(
    name_node: NameNode,
    array_dimensions: ArrayDimensions,
    opt_type_definition: Option<DimType>,
) -> Result<DimNameNode, QError> {
    let Locatable { element: name, pos } = name_node;
    let dim_type: DimType = match &name {
        Name::Bare(bare_name) => {
            map_bare_name_opt_extended_type_definition(bare_name, opt_type_definition)?
        }
        Name::Qualified(qualified_name) => {
            map_qualified_name_opt_extended_type_definition(qualified_name, opt_type_definition)?
        }
    };
    let final_dim_type = if array_dimensions.is_empty() {
        dim_type
    } else {
        DimType::Array(array_dimensions, Box::new(dim_type))
    };
    let bare_name: BareName = name.into();
    let dim_name = DimName::new(bare_name, final_dim_type);
    Ok(dim_name.at(pos))
}

fn map_bare_name_opt_extended_type_definition(
    bare_name: &BareName,
    opt_type_definition: Option<DimType>,
) -> Result<DimType, QError> {
    match opt_type_definition {
        Some(dim_type) => match dim_type {
            DimType::UserDefined(_) => {
                if bare_name.contains('.') {
                    Err(QError::IdentifierCannotIncludePeriod)
                } else {
                    Ok(dim_type)
                }
            }
            _ => Ok(dim_type),
        },
        None => Ok(DimType::Bare),
    }
}

fn map_qualified_name_opt_extended_type_definition(
    qualified_name: &QualifiedName,
    opt_type_definition: Option<DimType>,
) -> Result<DimType, QError> {
    if opt_type_definition.is_some() {
        Err(QError::syntax_error(
            "Identifier cannot end with %, &, !, #, or $",
        ))
    } else {
        let QualifiedName { qualifier, .. } = qualified_name;
        Ok(DimType::BuiltIn(*qualifier, BuiltInStyle::Compact))
    }
}
