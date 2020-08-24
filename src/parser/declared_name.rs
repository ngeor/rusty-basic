use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn declared_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DeclaredNameNode, QErrorNode>)> {
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
                ))
                .with_err_at(pos),
                None => Ok(DeclaredName::new(n, TypeDefinition::CompactBuiltIn(q)).at(pos)),
            },
        },
    )
}

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QErrorNode>)> {
    map_ng(
        with_some_whitespace_before_and_between(
            try_read_keyword(Keyword::As),
            extended_type(),
            || QError::SyntaxError("Expected type after AS".to_string()),
        ),
        |(_, r)| r,
    )
}

fn extended_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QErrorNode>)> {
    map_to_result_no_undo(
        with_pos(read_any_identifier()),
        |Locatable { element: x, pos }| match Keyword::from_str(&x) {
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
            ))
            .with_err_at(pos),
            Err(_) => Ok(TypeDefinition::UserDefined(x.into())),
        },
    )
}
