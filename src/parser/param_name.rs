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

enum TypeDefinition {
    ExtendedBuiltIn(TypeQualifier),
    UserDefined(BareName),
}

pub fn param_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamNode, QError>> {
    and_then(
        opt_seq2(with_pos(name::name()), type_definition_extended()),
        |(Locatable { element: name, pos }, opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(TypeDefinition::ExtendedBuiltIn(q)) => {
                    Ok(Param::ExtendedBuiltIn(b, q).at(pos))
                }
                Some(TypeDefinition::UserDefined(u)) => Ok(Param::UserDefined(b, u).at(pos)),
                None => Ok(Param::Bare(b).at(pos)),
            },
            Name::Qualified {
                name: n,
                qualifier: q,
            } => match opt_type_definition {
                Some(_) => Err(QError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                )),
                None => Ok(Param::Compact(n, q).at(pos)),
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
    and_then(
        with_pos(any_identifier()),
        |Locatable { element: x, .. }| match Keyword::from_str(&x) {
            Ok(Keyword::Single) => Ok(TypeDefinition::ExtendedBuiltIn(TypeQualifier::BangSingle)),
            Ok(Keyword::Double) => Ok(TypeDefinition::ExtendedBuiltIn(TypeQualifier::HashDouble)),
            Ok(Keyword::String_) => {
                Ok(TypeDefinition::ExtendedBuiltIn(TypeQualifier::DollarString))
            }
            Ok(Keyword::Integer) => Ok(TypeDefinition::ExtendedBuiltIn(
                TypeQualifier::PercentInteger,
            )),
            Ok(Keyword::Long) => Ok(TypeDefinition::ExtendedBuiltIn(
                TypeQualifier::AmpersandLong,
            )),
            Ok(_) => Err(QError::syntax_error(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
            )),
            Err(_) => {
                if x.len() > name::MAX_LENGTH {
                    Err(QError::IdentifierTooLong)
                } else {
                    Ok(TypeDefinition::UserDefined(x.into()))
                }
            }
        },
    )
}
