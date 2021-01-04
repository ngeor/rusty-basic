use std::marker::PhantomData;

use crate::common::*;
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::many::ManyParser;
use crate::parser::pc2::text::{string_while_p, whitespace_p, TextParser};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::{
    any_p, is_eol, is_eol_or_whitespace, item_p, Parser, Reader, ReaderResult, Undo,
};
use crate::parser::statement;
use crate::parser::statement::statement_p;
use crate::parser::types::*;

pub fn single_line_non_comment_statements_p<R>() -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(
            statement::single_line_non_comment_statement_p()
                .with_pos()
                .one_or_more_delimited_by(
                    item_p(':').surrounded_by_opt_ws(),
                    QError::syntax_error("Error: trailing colon"),
                ),
        )
        .keep_right()
}

pub fn single_line_statements_p<R>() -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(
            statement::single_line_statement_p()
                .with_pos()
                .one_or_more_delimited_by(
                    item_p(':').surrounded_by_opt_ws(),
                    QError::syntax_error("Error: trailing colon"),
                ),
        )
        .keep_right()
}

// When `zero_or_more_statements_p` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
pub fn zero_or_more_statements_p<R, S>(exit_source: S) -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + Undo<S::Output> + 'static,
    S: Parser<R>,
{
    // first separator
    // loop
    //      negate exit source
    //      statement node and separator
    StatementSeparator::new(false)
        .and(
            exit_source
                .negate()
                .and(StatementAndSeparator::new())
                .keep_right()
                .zero_or_more(),
        )
        .keep_right()
}

struct StatementAndSeparator<R>(PhantomData<R>);

impl<R> StatementAndSeparator<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Parser<R> for StatementAndSeparator<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = StatementNode;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_statement_node) = statement_p().with_pos().parse(reader)?;
        match opt_statement_node {
            Some(statement_node) => {
                let is_comment = if let Statement::Comment(_) = statement_node.as_ref() {
                    true
                } else {
                    false
                };
                let (reader, opt_separator) = StatementSeparator::new(is_comment).parse(reader)?;
                match opt_separator {
                    Some(_) => Ok((reader, Some(statement_node))),
                    _ => Err((reader, QError::syntax_error("Expected: end-of-statement"))),
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

struct StatementSeparator<R> {
    phantom_reader: PhantomData<R>,
    comment_mode: bool,
}

impl<R> StatementSeparator<R>
where
    R: Reader<Item = char, Err = QError>,
{
    pub fn new(comment_mode: bool) -> Self {
        Self {
            phantom_reader: PhantomData,
            comment_mode,
        }
    }

    fn parse_comment(&self, reader: R, mut buf: String) -> ReaderResult<R, String, R::Err> {
        let (reader, opt_item) = eol_separator_p().parse(reader)?;
        let item = opt_item.unwrap();
        buf.push_str(item.as_str());
        Ok((reader, Some(buf)))
    }

    // <ws>* '\'' (undoing it)
    // <ws>* ':' <ws*>
    // <ws>* EOL <ws | eol>*
    fn parse_non_comment(&self, reader: R, mut buf: String) -> ReaderResult<R, String, R::Err> {
        let (reader, opt_item) = comment_separator_p()
            .or(colon_separator_p())
            .or(eol_separator_p())
            .parse(reader)?;
        match opt_item {
            Some(item) => {
                buf.push_str(item.as_str());
                Ok((reader, Some(buf)))
            }
            _ => Err((reader, QError::syntax_error("Expected: end-of-statement"))),
        }
    }
}

impl<R> Parser<R> for StatementSeparator<R>
where
    R: Reader<Item = char, Err = QError>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        // skip any whitespace, so that the error will hit the first offending character
        let (reader, opt_buf) = whitespace_p().parse(reader)?;
        let buf = opt_buf.unwrap_or_default();
        if self.comment_mode {
            self.parse_comment(reader, buf)
        } else {
            self.parse_non_comment(reader, buf)
        }
    }
}

// '\'' (undoing it)
fn comment_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError>,
{
    item_p('\'').peek_reader_item().stringify()
}

// ':' <ws>*
fn colon_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError>,
{
    item_p(':').followed_by_opt_ws().stringify()
}

// <eol> < ws | eol >*
fn eol_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError>,
{
    any_p()
        .filter_reader_item(is_eol)
        .and_opt(string_while_p(is_eol_or_whitespace))
        .stringify()
}
