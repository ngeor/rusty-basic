// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::type_qualifier;

use crate::char_reader::*;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

pub fn declared_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DeclaredNameNode, QErrorNode>)> {
    with_pos(map_ng(
        if_first_demand_second(read_any_word(), type_definition(), || {
            QError::SyntaxError("Could not parse name".to_string())
        }),
        |(l, r)| DeclaredName::new(l.into(), r),
    ))
}

fn type_definition<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QErrorNode>)> {
    or_ng(type_definition_extended(), type_definition_compact())
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
                "Expected user defined type or standard type".to_string(),
            ))
            .with_err_at(pos),
            Err(_) => Ok(TypeDefinition::UserDefined(x.into())),
        },
    )
}

fn type_definition_compact<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeDefinition, QErrorNode>)> {
    or_ng(
        map_ng(type_qualifier::type_qualifier(), |q| {
            TypeDefinition::CompactBuiltIn(q)
        }),
        Box::new(move |reader| (reader, Ok(TypeDefinition::Bare))),
    )
}

pub fn take_if_declared_name<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<DeclaredNameNode>> {
    Box::new(switch_err(
        |(l, opt_r)| match opt_r {
            Some(r) => {
                let Locatable { element: name, pos } = l;
                match name {
                    Name::Bare(b) => Some(Ok(DeclaredName::new(b, r).at(pos))),
                    Name::Qualified { .. } => Some(
                        Err(QError::SyntaxError(
                            "Identifier cannot end with %, &, !, #, or $".to_string(),
                        ))
                        .with_err_at(pos),
                    ),
                }
            }
            None => Some(Ok(l.into_locatable())),
        },
        zip_allow_right_none(name::take_if_name_node(), as_part()),
    ))
}

fn as_part<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<TypeDefinition>> {
    apply(
        |(_, r)| r,
        with_leading_whitespace(with_whitespace_between(
            take_if_keyword(Keyword::As),
            take_and_map_to_result(lexeme_node_to_type_definition),
        )),
    )
}

fn lexeme_node_to_type_definition(lexeme_node: LexemeNode) -> Result<TypeDefinition, QErrorNode> {
    let Locatable { element, pos } = lexeme_node;
    let var_type = match element {
        Lexeme::Keyword(Keyword::Double, _) => {
            TypeDefinition::ExtendedBuiltIn(TypeQualifier::HashDouble)
        }
        Lexeme::Keyword(Keyword::Integer, _) => {
            TypeDefinition::ExtendedBuiltIn(TypeQualifier::PercentInteger)
        }
        Lexeme::Keyword(Keyword::Long, _) => {
            TypeDefinition::ExtendedBuiltIn(TypeQualifier::AmpersandLong)
        }
        Lexeme::Keyword(Keyword::Single, _) => {
            TypeDefinition::ExtendedBuiltIn(TypeQualifier::BangSingle)
        }
        Lexeme::Keyword(Keyword::String_, _) => {
            TypeDefinition::ExtendedBuiltIn(TypeQualifier::DollarString)
        }
        Lexeme::Word(w) => TypeDefinition::UserDefined(w.into()),
        _ => {
            return Err(QError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string(),
            ))
            .with_err_at(pos)
        }
    };
    Ok(var_type)
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<DeclaredNameNode>, QErrorNode> {
    take_if_declared_name()(lexer).transpose()
}
