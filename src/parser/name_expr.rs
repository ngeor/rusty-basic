use crate::common::QError;
use crate::parser::char_reader::EolReader;
use crate::parser::expression::expression_node;
use crate::parser::name::{any_word_with_dot, any_word_without_dot};
use crate::parser::pc::common::{and, drop_left, lazy, many, map_default_to_not_found, opt_seq4};
use crate::parser::pc::map::map;
use crate::parser::pc::{read, ReaderResult};
use crate::parser::pc_specific::{csv_zero_or_more, in_parenthesis};
use crate::parser::type_qualifier::type_qualifier;
use crate::parser::types::NameExpr;
use std::io::BufRead;

pub fn name_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameExpr, QError>> {
    map(
        opt_seq4(
            any_word_with_dot(),
            type_qualifier(),
            in_parenthesis(csv_zero_or_more(lazy(expression_node))),
            map_default_to_not_found(many(drop_left(and(read('.'), any_word_without_dot())))),
        ),
        |(name, qualifier, arguments, elements)| NameExpr {
            bare_name: name.into(),
            qualifier,
            arguments,
            elements,
        },
    )
}
