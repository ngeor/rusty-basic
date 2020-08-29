use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::try_read;
use crate::parser::pc::loc::with_pos;
use crate::parser::pc::str::{zero_or_more_if, zero_or_more_if_leading_remaining};
use crate::parser::pc::traits::*;
use crate::parser::pc::ws::{is_eol, is_eol_or_whitespace, is_whitespace};
use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

#[derive(Debug)]
pub struct ParseStatementsOptions {
    /// Indicates that a whitespace is a valid separator for the first statement in a block
    pub first_statement_separated_by_whitespace: bool,
    pub err: QError,
}

pub fn single_line_non_comment_statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(zero_or_more(opt_seq2(
        with_pos(statement::single_line_non_comment_statement()),
        crate::parser::pc::ws::zero_or_more_around(try_read(':')),
    ))))
}

pub fn single_line_statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(zero_or_more(opt_seq2(
        with_pos(statement::single_line_statement()),
        crate::parser::pc::ws::zero_or_more_around(try_read(':')),
    ))))
}

pub fn skip_until_first_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QError>)> {
    Box::new(move |reader| {
        let mut buf: String = String::new();

        // skip whitespace
        let (reader, res) = crate::parser::pc::ws::zero_or_more()(reader);
        match res {
            Err(err) => return (reader, Err(err)),
            Ok(x) => {
                buf.push_str(&x);
            }
        };

        let (reader, res) = reader.read();
        match res {
            Err(err) => (reader, Err(err)),
            Ok('\r') | Ok('\n') => {
                // take EOL, continue taking eol or whitespace
                buf.push('\n');
                map(zero_or_more_if(is_eol_or_whitespace), move |x| {
                    format!("{}{}", buf, x)
                })(reader)
            }
            Ok(':') => {
                // take colon separator, continue taking whitespace
                buf.push(':');
                map(crate::parser::pc::ws::zero_or_more(), move |x| {
                    format!("{}{}", buf, x)
                })(reader)
            }
            Ok(ch) => (reader.undo(ch), Ok(buf)),
        }
    })
}

pub fn statements<T: BufRead + 'static, S, X>(
    exit_source: S,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)>
where
    S: Fn(EolReader<T>) -> (EolReader<T>, Result<X, QError>) + 'static,
    EolReader<T>: Undo<X>,
{
    drop_left(and(
        skip_until_first_statement(),
        zero_or_more(move |reader| {
            let (reader, exit_result) = exit_source(reader);
            match exit_result {
                Ok(x) => {
                    // found the exit
                    reader.undo_and_err_not_found(x)
                }
                Err(err) => {
                    if err.is_not_found_err() {
                        // did not find the exit, we can parse a statement
                        statement_node_and_separator()(reader)
                    } else {
                        // something else happened, abort
                        (reader, Err(err))
                    }
                }
            }
        }),
    ))
}

fn statement_node_and_separator<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> (
        EolReader<T>,
        Result<(StatementNode, Option<String>), QError>,
    ),
> {
    Box::new(move |reader| {
        let (reader, statement_result) = statement::statement_node()(reader);
        match statement_result {
            Ok(s_node) => {
                // if the statement is a comment, the only valid separator is EOL (or EOF)
                let is_comment = match s_node.as_ref() {
                    Statement::Comment(_) => true,
                    _ => false,
                };
                let (reader, sep) = if is_comment {
                    comment_separator()(reader)
                } else {
                    crate::parser::pc::ws::zero_or_more_leading(non_comment_separator())(reader)
                };
                match sep {
                    Ok(x) => (reader, Ok((s_node, Some(x)))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (reader, Ok((s_node, None)))
                        } else {
                            (reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (reader, Err(err)),
        }
    })
}

fn comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QError>)> {
    map_default_to_not_found(zero_or_more_if_leading_remaining(
        is_eol,
        is_eol_or_whitespace,
    ))
}

fn non_comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QError>)> {
    // ws* : ws*
    // ws* eol (ws | eol)*
    // ws*' comment
    or_vec(vec![
        map_default_to_not_found(zero_or_more_if_leading_remaining(
            |ch| ch == ':',
            is_whitespace,
        )),
        comment_separator(),
        map_fully_ok(try_read('\''), |reader: EolReader<T>, c| {
            (reader.undo(c), Ok(format!("{}", c)))
        }),
    ])
}
