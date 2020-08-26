use super::{BareName, BareNameNode, Name, NameNode};
use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::loc::*;
use crate::parser::type_qualifier;
use std::io::BufRead;

// name node

pub fn name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<NameNode, QErrorNode>)> {
    with_pos(name())
}

pub fn name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Name, QErrorNode>)> {
    map(
        if_first_maybe_second(read_any_word(), type_qualifier::type_qualifier()),
        |(l, r)| Name::new(l.into(), r),
    )
}

// bare name node

pub fn bare_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<BareNameNode, QErrorNode>)> {
    with_pos(bare_name())
}

pub fn bare_name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<BareName, QErrorNode>)> {
    map(
        and(read_any_word(), negate(type_qualifier::type_qualifier())),
        |(l, _)| l.into(),
    )
}
