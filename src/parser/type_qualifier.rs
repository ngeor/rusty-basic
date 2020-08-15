use crate::common::*;
use crate::lexer::*;

use crate::parser::types::TypeQualifier;
use std::convert::TryFrom;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<TypeQualifier>, QErrorNode> {
    match lexer.peek_ref_ng()? {
        Some(Locatable {
            element: Lexeme::Symbol(ch),
            ..
        }) => match TypeQualifier::try_from(*ch) {
            Ok(t) => {
                lexer.read_ng()?;
                Ok(Some(t))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
