use super::{BareName, BareNameNode, Name, NameNode, ParserError};
use crate::common::*;
use crate::lexer::{BufLexer, LexemeNode};
use crate::parser::type_qualifier;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<NameNode>, ParserError> {
    match lexer.peek()? {
        LexemeNode::Word(word, pos) => {
            lexer.read()?;
            let q = type_qualifier::try_read(lexer)?;
            Ok(Some(Name::new(word, q).at(pos)))
        }
        _ => Ok(None),
    }
}

pub fn try_read_bare<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<BareNameNode>, ParserError> {
    lexer.begin_transaction();
    match lexer.peek()? {
        LexemeNode::Word(word, pos) => {
            lexer.read()?;

            // if we have a type qualifier next, then it's not a bare name actually
            match type_qualifier::try_read(lexer)? {
                Some(_) => {
                    lexer.rollback_transaction()?;
                    Ok(None)
                }
                None => {
                    lexer.commit_transaction()?;
                    Ok(Some(BareName::new(word).at(pos)))
                }
            }
        }
        _ => {
            lexer.rollback_transaction()?;
            Ok(None)
        }
    }
}
