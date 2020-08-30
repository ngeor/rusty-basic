use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::traits::*;
use crate::parser::types::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeQualifier, QError>)> {
    map_fully_ok(
        read_any_symbol(),
        |reader: EolReader<T>, ch| match TypeQualifier::from_char(ch) {
            Some(t) => (reader, Ok(t)),
            None => (reader.undo(ch), Err(QError::not_found_err())),
        },
    )
}
