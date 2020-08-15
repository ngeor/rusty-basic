use crate::common::*;
use crate::lexer::*;

use crate::parser::types::TypeQualifier;
use std::io::BufRead;

pub fn next<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<TypeQualifier, QErrorNode>> {
    lexer.map_if(lexeme_node_to_type_qualifier)
}

fn lexeme_node_to_type_qualifier(l: &LexemeNode) -> Option<Result<TypeQualifier, QErrorNode>> {
    let Locatable { element, .. } = l;
    match element {
        Lexeme::Symbol(ch) => TypeQualifier::from_char_ref(ch).map(|x| Ok(*x)),
        _ => None,
    }
}

#[deprecated]
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<TypeQualifier>, QErrorNode> {
    next(lexer).transpose()
}
