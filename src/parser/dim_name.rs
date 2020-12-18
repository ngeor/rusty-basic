use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::name::name_with_dot_p;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{and_then, map, source_and_then_some};
use crate::parser::pc::*;
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::Parser;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;
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

pub fn dim_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DimNameNode, QError>> {
    and_then(
        opt_seq3(
            name_with_dot_p().with_pos().convert_to_fn(),
            array_dimensions(),
            type_definition_extended(),
        ),
        |(name_node, opt_array_dimensions, opt_extended_type_definition)| {
            map_name_opt_extended_type_definition(
                name_node,
                opt_array_dimensions,
                opt_extended_type_definition,
            )
        },
    )
}

fn array_dimensions<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ArrayDimensions, QError>> {
    in_parenthesis(demand(
        map_default_to_not_found(csv_zero_or_more(array_dimension())),
        QError::syntax_error_fn("Expected: array dimension"),
    ))
}

fn array_dimension<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ArrayDimension, QError>> {
    map(
        opt_seq2(
            expression::expression_node(),
            drop_left(seq2(
                ws::one_or_more_leading(keyword(Keyword::To)),
                expression::demand_guarded_expression_node(),
            )),
        ),
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

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DimType, QError>> {
    // <ws+> AS <ws+> identifier
    drop_left(crate::parser::pc::ws::seq2(
        crate::parser::pc::ws::one_or_more_leading(keyword(Keyword::As)),
        demand(
            extended_type(),
            QError::syntax_error_fn("Expected: type after AS"),
        ),
        QError::syntax_error_fn("Expected: whitespace after AS"),
    ))
}

fn extended_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DimType, QError>> {
    source_and_then_some(
        with_pos(any_identifier_without_dot()),
        |reader, Locatable { element: x, pos }| match Keyword::from_str(&x) {
            Ok(Keyword::Single) => Ok((
                reader,
                Some(DimType::BuiltIn(
                    TypeQualifier::BangSingle,
                    BuiltInStyle::Extended,
                )),
            )),
            Ok(Keyword::Double) => Ok((
                reader,
                Some(DimType::BuiltIn(
                    TypeQualifier::HashDouble,
                    BuiltInStyle::Extended,
                )),
            )),
            Ok(Keyword::String_) => {
                let expr_res: ReaderResult<EolReader<T>, ExpressionNode, QError> =
                    drop_left(seq2(
                        ws::zero_or_more_around(read('*')),
                        expression::demand_expression_node(),
                    ))(reader);
                match expr_res {
                    Ok((reader, Some(e))) => Ok((reader, Some(DimType::FixedLengthString(e, 0)))),
                    Ok((reader, None)) => Ok((
                        reader,
                        Some(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Extended,
                        )),
                    )),
                    Err(err) => Err(err),
                }
            }
            Ok(Keyword::Integer) => Ok((
                reader,
                Some(DimType::BuiltIn(
                    TypeQualifier::PercentInteger,
                    BuiltInStyle::Extended,
                )),
            )),
            Ok(Keyword::Long) => Ok((
                reader,
                Some(DimType::BuiltIn(
                    TypeQualifier::AmpersandLong,
                    BuiltInStyle::Extended,
                )),
            )),
            Ok(_) => Err((
                reader,
                QError::syntax_error(
                    "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
                ),
            )),
            Err(_) => {
                if x.len() > name::MAX_LENGTH {
                    Err((reader, QError::IdentifierTooLong))
                } else {
                    let type_name: BareName = x.into();
                    Ok((reader, Some(DimType::UserDefined(type_name.at(pos)))))
                }
            }
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
