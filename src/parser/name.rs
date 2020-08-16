use super::{BareNameNode, Name, NameNode};
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::type_qualifier;
use std::io::BufRead;

// name node

pub fn take_if_name_node<T: BufRead>(
) -> impl Fn(&mut BufLexer<T>) -> Option<Result<NameNode, QErrorNode>> {
    apply(
        |(bare_name_node, opt_q)| bare_name_node.map(|n| Name::new(n, opt_q)),
        zip_allow_right_none(bare_name_node_parser_combinator(), type_qualifier::take_if_type_qualifier()),
    )
}

// bare name node

pub fn take_if_bare_name_node<T: BufRead>(
) -> impl Fn(&mut BufLexer<T>) -> Option<Result<BareNameNode, QErrorNode>> {
    in_transaction_pc(switch(
        |(bare_name_node, opt_q)| {
            if opt_q.is_none() {
                Some(Ok(bare_name_node))
            } else {
                // we specifically wanted a bare name,
                // but here we found a qualified name
                None
            }
        },
        zip_allow_right_none(bare_name_node_parser_combinator(), type_qualifier::take_if_type_qualifier()),
    ))
}

// private

fn bare_name_node_parser_combinator<T: BufRead>(
) -> impl Fn(&mut BufLexer<T>) -> Option<Result<BareNameNode, QErrorNode>> {
    map_from_locatable(take_if_any_word())
}

// deprecated

#[deprecated]
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<NameNode>, QErrorNode> {
    next(lexer).transpose()
}

#[deprecated]
pub fn next<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<NameNode, QErrorNode>> {
    take_if_name_node()(lexer)
}

#[deprecated]
pub fn next_bare<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<BareNameNode, QErrorNode>> {
    take_if_bare_name_node()(lexer)
}

#[deprecated]
pub fn try_read_bare<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<BareNameNode>, QErrorNode> {
    next_bare(lexer).transpose()
}
