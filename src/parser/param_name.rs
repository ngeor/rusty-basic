use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::map::and_then;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

// Parses a Param name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn param_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamNameNode, QError>> {
    and_then(
        opt_seq2(with_pos(name::name()), type_definition_extended()),
        |(Locatable { element: name, pos }, opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(param_type) => match param_type {
                    ParamType::UserDefined(_) => {
                        if b.contains('.') {
                            Err(QError::IdentifierCannotIncludePeriod)
                        } else {
                            Ok(ParamName::new(b, param_type).at(pos))
                        }
                    }
                    _ => Ok(ParamName::new(b, param_type).at(pos)),
                },
                None => Ok(ParamName::new(b, ParamType::Bare).at(pos)),
            },
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => match opt_type_definition {
                Some(_) => Err(QError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                )),
                None => Ok(ParamName::new(bare_name, ParamType::Compact(qualifier)).at(pos)),
            },
        },
    )
}

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamType, QError>> {
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamType, QError>> {
    and_then(
        with_pos(any_identifier()),
        |Locatable { element: x, pos }| match Keyword::from_str(&x) {
            Ok(Keyword::Single) => Ok(ParamType::Extended(TypeQualifier::BangSingle)),
            Ok(Keyword::Double) => Ok(ParamType::Extended(TypeQualifier::HashDouble)),
            Ok(Keyword::String_) => Ok(ParamType::Extended(TypeQualifier::DollarString)),
            Ok(Keyword::Integer) => Ok(ParamType::Extended(TypeQualifier::PercentInteger)),
            Ok(Keyword::Long) => Ok(ParamType::Extended(TypeQualifier::AmpersandLong)),
            Ok(_) => Err(QError::syntax_error(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
            )),
            Err(_) => {
                if x.len() > name::MAX_LENGTH {
                    Err(QError::IdentifierTooLong)
                } else {
                    let type_name: BareName = x.into();
                    Ok(ParamType::UserDefined(type_name.at(pos)))
                }
            }
        },
    )
}
