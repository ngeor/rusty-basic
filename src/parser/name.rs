use super::{BareName, BareNameNode, Name, NameNode};
use crate::common::*;
use crate::lexer::{BufLexer, Lexeme};
use crate::parser::type_qualifier;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<NameNode>, QErrorNode> {
    // TODO make helpers to improve this
    match lexer.peek_ng()? {
        Some(Locatable {
            element: Lexeme::Word(word),
            pos,
        }) => {
            let bare_name: BareName = word.clone().into();
            let name_pos = *pos;
            lexer.read_ng()?;
            let q = type_qualifier::try_read(lexer)?;
            Ok(Some(Name::new(bare_name, q).at(name_pos)))
        }
        _ => Ok(None),
    }
}

pub fn try_read_bare<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<BareNameNode>, QErrorNode> {
    lexer.begin_transaction();
    // TODO make helpers to improve this
    match lexer.peek_ng()? {
        Some(Locatable {
            element: Lexeme::Word(word),
            pos,
        }) => {
            let bare_name: BareName = word.clone().into();
            let name_pos = *pos;
            lexer.read_ng()?;

            // if we have a type qualifier next, then it's not a bare name actually
            match type_qualifier::try_read(lexer)? {
                Some(_) => {
                    lexer.rollback_transaction();
                    Ok(None)
                }
                None => {
                    lexer.commit_transaction();
                    Ok(Some(bare_name.at(name_pos)))
                }
            }
        }
        _ => {
            lexer.rollback_transaction();
            Ok(None)
        }
    }
}
