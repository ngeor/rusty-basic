use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::{peek, try_read};
use crate::parser::pc::loc::with_pos;
use crate::parser::pc::str::{map_to_str, zero_or_more_if_leading_remaining};
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

fn parse_first_statement_separator<T: BufRead + 'static, FE>(
    err_fn: FE,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QError>)>
where
    FE: Fn() -> QError + 'static,
{
    // Allowed to read:
    // <ws*> ' -- and if it is comment, put it back
    // <ws*> :
    // <ws*> EOL
    Box::new(move |reader| {
        let mut r = reader;
        let mut buf: String = String::new();
        let mut found = false;
        loop {
            let (reader, res) = r.read();
            r = reader;
            match res {
                Ok(ch) => {
                    if crate::parser::pc::ws::is_whitespace(ch) {
                        if found {
                            buf.push(ch);
                        } else {
                            // skip over it, so that the error will hit the
                            // first non-whitespace character
                        }
                    } else if ch == ':' || crate::parser::pc::ws::is_eol(ch) {
                        // exit
                        buf.push(ch);
                        found = true;
                    } else if ch == '\'' {
                        return (r.undo(ch), Ok(buf));
                    } else {
                        r = r.undo(ch);
                        break;
                    }
                }
                Err(err) => {
                    if err.is_not_found_err() {
                        break;
                    } else {
                        return (r, Err(err));
                    }
                }
            }
        }
        if found {
            (r, Ok(buf))
        } else {
            (r.undo(buf), Err(err_fn()))
        }
    })
}

// When `statements` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
pub fn statements<T: BufRead + 'static, S, X, FE>(
    exit_source: S,
    err_fn: FE,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)>
where
    S: Fn(EolReader<T>) -> (EolReader<T>, Result<X, QError>) + 'static,
    EolReader<T>: Undo<X>,
    FE: Fn() -> QError + 'static,
{
    drop_left(and(
        parse_first_statement_separator(err_fn),
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
        map_to_str(peek('\'')),
        crate::parser::pc::ws::zero_or_more_leading(map_fully_ok(
            read(),
            // undo so that the error will be positioned at the offending character
            |reader: EolReader<T>, ch| {
                (
                    reader.undo(ch),
                    Err(QError::syntax_error("Expected: end-of-statement")),
                )
            },
        )),
    ])
}
