use crate::common::QError;
use crate::parser::char_reader::EolReader;
use crate::parser::expression::expression_node;
use crate::parser::pc::common::{lazy, opt_seq2};
use crate::parser::pc::map::{and_then, map, source_and_then_some};
use crate::parser::pc::{ReaderResult, Undo};
use crate::parser::pc_specific::{any_identifier_without_dot, csv_zero_or_more, in_parenthesis};
use crate::parser::types::NameExpr;
use crate::parser::Keyword;
use std::io::BufRead;
use std::str::FromStr;

pub fn name_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameExpr, QError>> {
    map(
        opt_seq2(
            any_word_without_dot(),
            in_parenthesis(csv_zero_or_more(lazy(expression_node))),
        ),
        |(name, args)| NameExpr {
            bare_name: name.into(),
            qualifier: None,
            arguments: args,
            elements: None,
        },
    )
}

pub const MAX_LENGTH: usize = 40;

/// Reads any word, i.e. any identifier which is not a keyword.
fn any_word_without_dot<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, String, QError>> {
    source_and_then_some(any_identifier_without_dot(), map_word_without_dot)
}

fn map_word_without_dot<T: BufRead + 'static>(
    reader: EolReader<T>,
    s: String,
) -> ReaderResult<EolReader<T>, String, QError> {
    if s.len() > MAX_LENGTH {
        Err((reader, QError::IdentifierTooLong))
    } else {
        match Keyword::from_str(&s) {
            Ok(_) => Ok((reader.undo(s), None)),
            Err(_) => Ok((reader, Some(s))),
        }
    }
}
