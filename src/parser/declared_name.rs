// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::error::*;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<DeclaredNameNode>, ParserError> {
    if !lexer.peek()?.as_ref().is_word() {
        return Ok(None);
    }

    // demand variable name
    let var_name_node = demand(lexer, name::try_read, "Expected variable name")?;
    let is_long = in_transaction(lexer, |lexer| {
        read_whitespace(lexer)?;
        read_keyword(lexer, Keyword::As)?;
        read_whitespace(lexer)
    })?
    .is_some();
    if !is_long {
        return Ok(Some(var_name_node.into_locatable()));
    }
    // explicit type requires a bare name
    let bare_name = match var_name_node.as_ref() {
        Name::Bare(b) => b.clone(),
        _ => {
            return Err(ParserError::SyntaxError(
                "Identifier cannot end with %, &, !, #, or $".to_string(),
                var_name_node.pos(),
            ));
        }
    };
    // demand type name
    let next = lexer.read()?;
    let var_type = match next.as_ref() {
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
        Lexeme::Word(w) => TypeDefinition::UserDefined(w.clone().into()),
        _ => {
            return Err(ParserError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string(),
                next.pos(),
            ))
        }
    };
    Ok(Some(
        DeclaredName::new(bare_name, var_type).at(var_name_node.pos()),
    ))
}
