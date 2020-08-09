use crate::common::*;
use crate::lexer::*;

use crate::parser::types::TypeQualifier;
use std::convert::TryFrom;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<TypeQualifier>, QErrorNode> {
    match lexer.peek()? {
        Locatable {
            element: Lexeme::Symbol(ch),
            ..
        } => match TypeQualifier::try_from(ch) {
            Ok(t) => {
                lexer.read()?;
                Ok(Some(t))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
