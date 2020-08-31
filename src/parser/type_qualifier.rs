use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::*;
use crate::parser::types::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeQualifier, QError>> {
    map_fully_ok(
        read_any_symbol(),
        |reader: EolReader<T>, ch| match TypeQualifier::from_char(ch) {
            Some(t) => Ok((reader, Some(t))),
            None => Ok((reader.undo(ch), None)),
        },
    )
}
