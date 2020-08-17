use crate::common::*;
use crate::lexer::*;

use crate::parser::types::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn take_if_type_qualifier<T: BufRead>() -> impl Fn(&mut BufLexer<T>) -> OptRes<TypeQualifier> {
    take_if_map(mapper)
}

fn mapper(l: LexemeNode) -> Option<TypeQualifier> {
    let Locatable { element, .. } = l;
    match element {
        Lexeme::Symbol(ch) => TypeQualifier::from_char(ch),
        _ => None,
    }
}
