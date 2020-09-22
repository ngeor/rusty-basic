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

enum TypeDefinition {
    ExtendedBuiltIn(TypeQualifier),
    UserDefined(BareName),
    String(ExpressionNode),
}

pub fn dim_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DimNameNode, QError>> {
    and_then(
        opt_seq2(with_pos(name::name()), type_definition_extended()),
        |(Locatable { element: name, pos }, opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(TypeDefinition::ExtendedBuiltIn(q)) => {
                    Ok(DimName::ExtendedBuiltIn(b, q).at(pos))
                }
                Some(TypeDefinition::String(e)) => Ok(DimName::String(b, e).at(pos)),
                Some(TypeDefinition::UserDefined(u)) => Ok(DimName::UserDefined(b, u).at(pos)),
                None => Ok(DimName::Bare(b).at(pos)),
            },
            Name::Qualified {
                name: n,
                qualifier: q,
            } => match opt_type_definition {
                Some(_) => Err(QError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                )),
                None => Ok(DimName::Compact(n, q).at(pos)),
            },
        },
    )
}

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeDefinition, QError>> {
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeDefinition, QError>> {
    source_and_then_some(
        with_pos(any_identifier()),
        |reader, Locatable { element: x, .. }| match Keyword::from_str(&x) {
            Ok(Keyword::Single) => Ok((
                reader,
                Some(TypeDefinition::ExtendedBuiltIn(TypeQualifier::BangSingle)),
            )),
            Ok(Keyword::Double) => Ok((
                reader,
                Some(TypeDefinition::ExtendedBuiltIn(TypeQualifier::HashDouble)),
            )),
            Ok(Keyword::String_) => {
                let expr_res: ReaderResult<EolReader<T>, ExpressionNode, QError> =
                    drop_left(seq2(
                        ws::zero_or_more_around(read('*')),
                        demand_expression_node(),
                    ))(reader);
                match expr_res {
                    Ok((reader, Some(e))) => Ok((reader, Some(TypeDefinition::String(e)))),
                    Ok((reader, None)) => Ok((
                        reader,
                        Some(TypeDefinition::ExtendedBuiltIn(TypeQualifier::DollarString)),
                    )),
                    Err(err) => Err(err),
                }
            }
            Ok(Keyword::Integer) => Ok((
                reader,
                Some(TypeDefinition::ExtendedBuiltIn(
                    TypeQualifier::PercentInteger,
                )),
            )),
            Ok(Keyword::Long) => Ok((
                reader,
                Some(TypeDefinition::ExtendedBuiltIn(
                    TypeQualifier::AmpersandLong,
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
                    Ok((reader, Some(TypeDefinition::UserDefined(x.into()))))
                }
            }
        },
    )
}
