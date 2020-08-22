use crate::common::*;
use crate::lexer::*;

use crate::char_reader::*;
use crate::parser::types::TypeQualifier;
use std::io::BufRead;

pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeQualifier, QErrorNode>)> {
    map_or_undo(read_any_symbol(), |ch| match TypeQualifier::from_char(ch) {
        Some(t) => MapOrUndo::Ok(t),
        None => MapOrUndo::Undo(ch),
    })
}

/// Returns a function that can parse a `TypeQualifier`.
#[deprecated]
pub fn take_if_type_qualifier<T: BufRead>() -> impl Fn(&mut BufLexer<T>) -> OptRes<TypeQualifier> {
    take_if_map(mapper)
}

#[deprecated]
fn mapper(l: LexemeNode) -> Option<TypeQualifier> {
    let Locatable { element, .. } = l;
    match element {
        Lexeme::Symbol(ch) => TypeQualifier::from_char(ch),
        _ => None,
    }
}
