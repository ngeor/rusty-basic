use super::{BareName, BareNameNode, Name, NameNode};
use crate::common::*;
use crate::lexer::*;
use crate::parser::type_qualifier;
use std::io::BufRead;

#[deprecated]
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<NameNode>, QErrorNode> {
    next(lexer).transpose()
}

pub fn next<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<NameNode, QErrorNode>> {
    lexer
        .map_if(lexeme_node_to_bare_name_node)
        .zip_allow_right_none(|| type_qualifier::next(lexer))
        .map_ok(|(bare_name_node, opt_q)| bare_name_node.map(|n| Name::new(n, opt_q)))
}

pub fn next_bare<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<BareNameNode, QErrorNode>> {
    lexer.in_transaction_opt(|x| {
        x.map_if(lexeme_node_to_bare_name_node)
            .zip_allow_right_none(|| type_qualifier::next(x))
            .and_then_ok(|(bare_name_node, opt_q)| {
                if opt_q.is_none() {
                    Some(Ok(bare_name_node))
                } else {
                    // we specifically wanted a bare name,
                    // but here we found a qualified name
                    None
                }
            })
    })
}

fn lexeme_node_to_bare_name_node(l: &LexemeNode) -> Option<Result<BareNameNode, QErrorNode>> {
    let Locatable { element, pos } = l;
    match element {
        Lexeme::Word(word) => {
            let bare_name: BareName = word.clone().into();
            let name_pos = *pos;
            Some(Ok(bare_name.at(name_pos)))
        }
        _ => None,
    }
}

#[deprecated]
pub fn try_read_bare<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<BareNameNode>, QErrorNode> {
    next_bare(lexer).transpose()
}
