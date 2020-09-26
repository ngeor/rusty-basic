use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression::demand_expression_node;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{and_then, source_and_then_some};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn dim_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DimNameNode, QError>> {
    and_then(
        opt_seq2(with_pos(name::name()), type_definition_extended()),
        |(Locatable { element: name, pos }, opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(dim_type) => match dim_type {
                    DimType::UserDefined(_) => {
                        if b.contains('.') {
                            Err(QError::IdentifierCannotIncludePeriod)
                        } else {
                            Ok(DimName::new(b, dim_type).at(pos))
                        }
                    }
                    _ => Ok(DimName::new(b, dim_type).at(pos)),
                },
                None => Ok(DimName::new(b, DimType::Bare).at(pos)),
            },
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => match opt_type_definition {
                Some(_) => Err(QError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                )),
                None => Ok(DimName::new(bare_name, DimType::Compact(qualifier)).at(pos)),
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
        with_pos(any_identifier()),
        |reader, Locatable { element: x, pos }| match Keyword::from_str(&x) {
            Ok(Keyword::Single) => Ok((reader, Some(DimType::Extended(TypeQualifier::BangSingle)))),
            Ok(Keyword::Double) => Ok((reader, Some(DimType::Extended(TypeQualifier::HashDouble)))),
            Ok(Keyword::String_) => {
                let expr_res: ReaderResult<EolReader<T>, ExpressionNode, QError> =
                    drop_left(seq2(
                        ws::zero_or_more_around(read('*')),
                        demand_expression_node(),
                    ))(reader);
                match expr_res {
                    Ok((reader, Some(e))) => Ok((reader, Some(DimType::FixedLengthString(e)))),
                    Ok((reader, None)) => {
                        Ok((reader, Some(DimType::Extended(TypeQualifier::DollarString))))
                    }
                    Err(err) => Err(err),
                }
            }
            Ok(Keyword::Integer) => Ok((
                reader,
                Some(DimType::Extended(TypeQualifier::PercentInteger)),
            )),
            Ok(Keyword::Long) => Ok((
                reader,
                Some(DimType::Extended(TypeQualifier::AmpersandLong)),
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
