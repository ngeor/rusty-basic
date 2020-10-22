use super::{BareName, BareNameNode, Keyword, Name, NameNode};
use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{map, source_and_then_some};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::type_qualifier;
use std::io::BufRead;
use std::str::FromStr;

// name node

pub fn name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameNode, QError>> {
    with_pos(name())
}

pub fn name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Name, QError>> {
    map(
        opt_seq2(any_word_with_dot(), type_qualifier::type_qualifier()),
        |(l, r)| Name::new(l, r),
    )
}

// bare name node

pub fn bare_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareNameNode, QError>> {
    with_pos(bare_name())
}

pub fn bare_name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    drop_right(and(
        any_word_with_dot(),
        negate(type_qualifier::type_qualifier()),
    ))
}

pub const MAX_LENGTH: usize = 40;

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_with_dot<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    source_and_then_some(any_identifier_with_dot(), ensure_length_and_not_keyword)
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_without_dot<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    source_and_then_some(any_identifier_without_dot(), ensure_length_and_not_keyword)
}

fn ensure_length_and_not_keyword<T: BufRead + 'static>(
    reader: EolReader<T>,
    s: String,
) -> ReaderResult<EolReader<T>, BareName, QError> {
    if s.len() > MAX_LENGTH {
        Err((reader, QError::IdentifierTooLong))
    } else {
        match Keyword::from_str(&s) {
            Ok(_) => Ok((reader.undo(s), None)),
            Err(_) => Ok((reader, Some(s.into()))),
        }
    }
}
