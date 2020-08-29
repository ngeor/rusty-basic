use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::loc::*;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn declared_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DeclaredNameNode, QError>)> {
    map_to_result_no_undo(
        if_first_maybe_second(with_pos(name::name()), type_definition_extended()),
        |(Locatable { element: name, pos }, opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(t) => Ok(DeclaredName::new(b, t).at(pos)),
                None => Ok(DeclaredName::new(b, TypeDefinition::Bare).at(pos)),
            },
            Name::Qualified {
                name: n,
                qualifier: q,
            } => match opt_type_definition {
                Some(_) => Err(QError::SyntaxError(
                    "Identifier cannot end with %, &, !, #, or $".to_string(),
                )),
                None => Ok(DeclaredName::new(n, TypeDefinition::CompactBuiltIn(q)).at(pos)),
            },
        },
    )
}

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::As)),
        demand(
            extended_type(),
            QError::syntax_error_fn("Expected type after AS"),
        ),
        QError::syntax_error_fn("Expected whitespace after AS"),
    ))
}

fn extended_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QError>)> {
    map_to_result_no_undo(
        with_pos(read_any_identifier()),
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
            Ok(_) => Err(QError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string(),
            )),
            Err(_) => Ok(TypeDefinition::UserDefined(x.into())),
        },
    )
}
