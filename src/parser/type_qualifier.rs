use crate::common::*;
use crate::lexer::*;

use crate::parser::types::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn take_if_type_qualifier<T: BufRead>() -> impl Fn(&mut BufLexer<T>) -> Option<Result<TypeQualifier, QErrorNode>>
{
    take_if(predicate, mapper)
}

fn predicate(l: &LexemeNode) -> bool {
    let Locatable { element, .. } = l;
    match element {
        Lexeme::Symbol(ch) => TypeQualifier::from_char_ref(ch).is_some(),
        _ => false,
    }
}

fn mapper(l: LexemeNode) -> Option<TypeQualifier> {
    let Locatable { element, .. } = l;
    match element {
        Lexeme::Symbol(ch) => TypeQualifier::from_char_ref(&ch).map(|x| *x),
        _ => None,
    }
}

#[deprecated]
pub fn next<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<TypeQualifier, QErrorNode>> {
    take_if_type_qualifier()(lexer)
}

#[deprecated]
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<TypeQualifier>, QErrorNode> {
    next(lexer).transpose()
}
