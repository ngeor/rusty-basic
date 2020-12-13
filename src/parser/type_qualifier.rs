use crate::common::QError;
use crate::parser::char_reader::EolReader;
use crate::parser::pc::{Reader, ReaderResult};
use crate::parser::pc2::{read_one_if_try_from_p, Parser, ReadOneIfTryFrom};
use crate::parser::TypeQualifier;
use std::io::BufRead;

/// Returns a function that can parse a `TypeQualifier`.
#[deprecated]
pub fn type_qualifier<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeQualifier, QError>> {
    type_qualifier_p().convert_to_fn()
}

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> ReadOneIfTryFrom<R, TypeQualifier>
where
    R: Reader<Item = char>,
{
    read_one_if_try_from_p::<R, TypeQualifier>()
}
