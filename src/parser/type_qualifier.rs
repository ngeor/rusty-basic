use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::map::source_and_then_some;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::TypeQualifier;
use std::convert::TryFrom;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeQualifier, QError>> {
    source_and_then_some(
        any_symbol(),
        |reader: EolReader<T>, ch| match TypeQualifier::try_from(ch) {
            Ok(t) => Ok((reader, Some(t))),
            Err(_) => Ok((reader.undo(ch), None)),
        },
    )
}
