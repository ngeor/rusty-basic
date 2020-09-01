use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::{peek, try_read};
use crate::parser::pc::loc::with_pos;
use crate::parser::pc::map::source_and_then_some;
use crate::parser::pc::str::{map_to_str, zero_or_more_if_leading_remaining};
use crate::parser::pc::ws::{is_eol, is_eol_or_whitespace, is_whitespace};
use crate::parser::pc::*;
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>> {
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(zero_or_more(opt_seq2(
        with_pos(statement::single_line_non_comment_statement()),
        crate::parser::pc::ws::zero_or_more_around(try_read(':')),
    ))))
}

pub fn single_line_statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>> {
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(zero_or_more(opt_seq2(
        with_pos(statement::single_line_statement()),
        crate::parser::pc::ws::zero_or_more_around(try_read(':')),
    ))))
}

fn parse_first_statement_separator<T: BufRead + 'static, FE>(
    err_fn: FE,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, String, QError>>
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
            match r.read() {
                Ok((tmp, Some(ch))) => {
                    r = tmp;
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
                        return Ok((r.undo(ch), Some(buf)));
                    } else {
                        r = r.undo(ch);
                        break;
                    }
                }
                Ok((tmp, None)) => {
                    r = tmp;
                    break;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        if found {
            Ok((r, Some(buf)))
        } else {
            Err((r, err_fn()))
        }
    })
}

// When `statements` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
pub fn statements<T: BufRead + 'static, S, X, FE>(
    exit_source: S,
    err_fn: FE,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>>
where
    S: Fn(EolReader<T>) -> ReaderResult<EolReader<T>, X, QError> + 'static,
    EolReader<T>: Undo<X>,
    FE: Fn() -> QError + 'static,
{
    drop_left(and(
        parse_first_statement_separator(err_fn),
        zero_or_more(move |reader| {
            match exit_source(reader) {
                Ok((reader, Some(x))) => {
                    // found the exit
                    Ok((reader.undo(x), None))
                }
                Ok((reader, None)) => {
                    // did not find the exit, we can parse a statement
                    statement_node_and_separator()(reader)
                }
                Err(err) => {
                    // something else happened, abort
                    Err(err)
                }
            }
        }),
    ))
}

fn statement_node_and_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (StatementNode, Option<String>), QError>>
{
    Box::new(move |reader| {
        match statement::statement_node()(reader) {
            Ok((reader, opt_res)) => {
                match opt_res {
                    Some(s_node) => {
                        // if the statement is a comment, the only valid separator is EOL (or EOF)
                        let is_comment = match s_node.as_ref() {
                            Statement::Comment(_) => true,
                            _ => false,
                        };
                        let next = if is_comment {
                            comment_separator()(reader)
                        } else {
                            crate::parser::pc::ws::zero_or_more_leading(non_comment_separator())(
                                reader,
                            )
                        };
                        match next {
                            Ok((reader, opt_next)) => match opt_next {
                                Some(x) => Ok((reader, Some((s_node, Some(x))))),
                                None => Ok((reader, Some((s_node, None)))),
                            },
                            Err(err) => Err(err),
                        }
                    }
                    None => Ok((reader, None)),
                }
            }
            Err(err) => Err(err),
        }
    })
}

fn comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, String, QError>> {
    map_default_to_not_found(zero_or_more_if_leading_remaining(
        is_eol,
        is_eol_or_whitespace,
    ))
}

fn non_comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, String, QError>> {
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
        crate::parser::pc::ws::zero_or_more_leading(source_and_then_some(
            read(),
            |reader: EolReader<T>, ch| {
                // undo so that the error will be positioned at the offending character
                Err((
                    reader.undo(ch),
                    QError::syntax_error("Expected: end-of-statement"),
                ))
            },
        )),
    ])
}
