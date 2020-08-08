use crate::lexer::*;
use crate::parser::error::*;
use crate::parser::types::TypeQualifier;
use std::convert::TryFrom;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<TypeQualifier>, ParserError> {
    match lexer.peek()? {
        LexemeNode::Symbol(ch, _) => match TypeQualifier::try_from(ch) {
            Ok(t) => {
                lexer.read()?;
                Ok(Some(t))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
