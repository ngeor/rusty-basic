use crate::common::*;
use crate::parser::char_reader::*;
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
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)> {
    map(
        and(
            read_any_whitespace(),
            map(
                take_zero_or_more(
                    if_first_maybe_second(
                        with_pos(statement::single_line_non_comment_statement()),
                        skipping_whitespace_around(try_read_char(':')),
                    ),
                    |x| x.1.is_none(),
                ),
                |x| x.into_iter().map(|i| i.0).collect(),
            ),
        ),
        |(_, r)| r,
    )
}

pub fn single_line_statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)> {
    map(
        and(
            read_any_whitespace(),
            map(
                take_zero_or_more(
                    if_first_maybe_second(
                        with_pos(statement::single_line_statement()),
                        skipping_whitespace_around(try_read_char(':')),
                    ),
                    |x| x.1.is_none(),
                ),
                |x| x.into_iter().map(|i| i.0).collect(),
            ),
        ),
        |(_, r)| r,
    )
}

pub fn skip_until_first_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>)> {
    Box::new(move |reader| {
        let mut buf: String = String::new();
        let (reader, res) = skip_whitespace()(reader);
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
                buf.push('\n');
                map(skip_whitespace_eol(), move |x| format!("{}{}", buf, x))(reader)
            }
            Ok(':') => {
                buf.push(':');
                map(skip_whitespace(), move |x| format!("{}{}", buf, x))(reader)
            }
            Ok(ch) => (reader.undo(ch), Ok(buf)),
        }
    })
}

pub fn statements<T: BufRead + 'static, S, X>(
    exit_source: S,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)>
where
    S: Fn(EolReader<T>) -> (EolReader<T>, Result<X, QErrorNode>) + 'static,
    EolReader<T>: Undo<X>,
{
    map(
        maybe_first_and_second_no_undo(
            skip_until_first_statement(),
            take_zero_or_more_to_default(
                move |reader| {
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
                },
                |x: &(StatementNode, String)| x.1.is_empty(),
            ),
        ),
        |(_, x)| x.into_iter().map(|x| x.0).collect(),
    )
}

fn statement_node_and_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<(StatementNode, String), QErrorNode>)> {
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
                    skipping_whitespace(non_comment_separator())(reader)
                };
                match sep {
                    Ok(x) => (reader, Ok((s_node, x))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (reader, Ok((s_node, String::new())))
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

pub fn comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>)> {
    map(
        if_first_maybe_second(read_any_eol(), read_any_eol_whitespace()),
        |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
    )
}

pub fn non_comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>)> {
    // ws* : ws*
    // ws* eol (ws | eol)*
    // ws*' comment
    or_vec(vec![
        map(
            if_first_maybe_second(try_read_char(':'), read_any_whitespace()),
            |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
        ),
        map(
            if_first_maybe_second(read_any_eol(), read_any_eol_whitespace()),
            |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
        ),
        map(undo_if_ok(try_read_char('\'')), |c| format!("{}", c)),
    ])
}
