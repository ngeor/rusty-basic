// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;

use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;

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
    Box::new(apply(
        |(_, r)| r,
        with_leading_space(with_space_between(
            take_if_keyword(Keyword::As),
            take_and_map_to_result(lexeme_node_to_type_definition),
        )),
    ))
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
