use crate::common::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::name::name_with_dot_p;
use crate::parser::pc::*;
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::text::{whitespace_p, TextParser};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::{item_p, static_err_p, static_p, Parser};
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::str::FromStr;

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

pub fn dim_name_node_p<R>() -> impl Parser<R, Output = DimNameNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    name_with_dot_p()
        .with_pos()
        .and_opt(array_dimensions_p())
        .and_opt(type_definition_extended_p())
        .and_then(
            |((name_node, opt_array_dimensions), opt_extended_type_definition)| {
                map_name_opt_extended_type_definition(
                    name_node,
                    opt_array_dimensions,
                    opt_extended_type_definition,
                )
            },
        )
}

fn array_dimensions_p<R>() -> impl Parser<R, Output = ArrayDimensions>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    in_parenthesis_p(
        array_dimension_p()
            .csv()
            .or_syntax_error("Expected: array dimension"),
    )
}

fn array_dimension_p<R>() -> impl Parser<R, Output = ArrayDimension>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::expression_node_p()
        .and_opt(
            whitespace_p()
                .and(keyword_p(Keyword::To))
                .and_demand(
                    expression::guarded_expression_node_p()
                        .or_syntax_error("Expected: expression after TO"),
                )
                .keep_right(),
        )
        .map(|(l, opt_r)| match opt_r {
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

fn type_definition_extended_p<R>() -> impl Parser<R, Output = DimType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // <ws+> AS <ws+> identifier
    whitespace_p()
        .and(keyword_p(Keyword::As))
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after AS"))
        .and_demand(extended_type_p().or_syntax_error("Expected: type after AS"))
        .keep_right()
}

fn extended_type_p<R>() -> impl Parser<R, Output = DimType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    identifier_without_dot_p()
        .with_pos()
        .switch(
            |Locatable { element: x, pos }| match Keyword::from_str(&x) {
                Ok(Keyword::Single) => static_p(DimType::BuiltIn(
                    TypeQualifier::BangSingle,
                    BuiltInStyle::Extended,
                ))
                .box_dyn(),
                Ok(Keyword::Double) => static_p(DimType::BuiltIn(
                    TypeQualifier::HashDouble,
                    BuiltInStyle::Extended,
                ))
                .box_dyn(),
                Ok(Keyword::String_) => item_p('*')
                    .surrounded_by_opt_ws()
                    .and_demand(
                        expression::expression_node_p()
                            .or_syntax_error("Expected: string length after *"),
                    )
                    .keep_right()
                    .optional()
                    .map(|opt_len| match opt_len {
                        Some(l) => DimType::FixedLengthString(l, 0),
                        _ => DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
                    })
                    .box_dyn(),
                Ok(Keyword::Integer) => static_p(DimType::BuiltIn(
                    TypeQualifier::PercentInteger,
                    BuiltInStyle::Extended,
                ))
                .box_dyn(),
                Ok(Keyword::Long) => static_p(DimType::BuiltIn(
                    TypeQualifier::AmpersandLong,
                    BuiltInStyle::Extended,
                ))
                .box_dyn(),
                Ok(_) => static_err_p(QError::syntax_error(
                    "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
                ))
                .box_dyn(),
                Err(_) => static_p(x)
                    .validate(|x| {
                        if x.len() > name::MAX_LENGTH {
                            Err(QError::IdentifierTooLong)
                        } else {
                            Ok(true)
                        }
                    })
                    .map(move |x| DimType::UserDefined(BareName::from(x).at(pos)))
                    .box_dyn(),
            },
        )
}

fn map_name_opt_extended_type_definition(
    name_node: NameNode,
    opt_array_dimensions: Option<ArrayDimensions>,
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
    let final_dim_type = match opt_array_dimensions {
        Some(array_dimensions) => DimType::Array(array_dimensions, Box::new(dim_type)),
        _ => dim_type,
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
