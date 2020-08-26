use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::types::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeQualifier, QError>)> {
    map_or_undo(read_any_symbol(), |ch| match TypeQualifier::from_char(ch) {
        Some(t) => MapOrUndo::Ok(t),
        None => MapOrUndo::Undo(ch),
    })
}
