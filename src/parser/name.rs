use super::{BareName, BareNameNode, Name, NameNode};
use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::loc::*;
use crate::parser::pc::map::map;
use crate::parser::pc::misc::*;
use crate::parser::pc::*;
use crate::parser::type_qualifier;
use std::io::BufRead;

// name node

pub fn name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameNode, QError>> {
    with_pos(name())
}

pub fn name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Name, QError>> {
    map(
        opt_seq2(read_any_word(), type_qualifier::type_qualifier()),
        |(l, r)| Name::new(l.into(), r),
    )
}

// bare name node

pub fn bare_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareNameNode, QError>> {
    with_pos(bare_name())
}

pub fn bare_name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    map(
        and(read_any_word(), negate(type_qualifier::type_qualifier())),
        |(l, _)| l.into(),
    )
}
