use crate::char_reader::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::comment;
use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

#[derive(Debug)]
pub struct ParseStatementsOptions {
    /// Indicates that a whitespace is a valid separator for the first statement in a block
    pub first_statement_separated_by_whitespace: bool,
    pub err: QError,
}

pub fn statements<T: BufRead + 'static, S, X>(
    exit_source: S,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)>
where
    S: Fn(EolReader<T>) -> (EolReader<T>, Result<X, QErrorNode>) + 'static,
    EolReader<T>: Undo<X>,
{
    map_ng(
        take_zero_or_more(
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
        |x| x.into_iter().map(|x| x.0).collect(),
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
                    non_comment_separator()(reader)
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
    map_ng(
        if_first_maybe_second(read_any_eol(), read_any_eol_whitespace()),
        |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
    )
}

pub fn non_comment_separator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>)> {
    // ws* : ws*
    // ws* eol (ws | eol)*
    or_ng(
        map_ng(
            if_first_maybe_second(
                skipping_whitespace_ng(try_read_char(':')),
                read_any_whitespace(),
            ),
            |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
        ),
        map_ng(
            if_first_maybe_second(
                skipping_whitespace_ng(read_any_eol()),
                read_any_eol_whitespace(),
            ),
            |(l, r)| format!("{}{}", l, r.unwrap_or_default()),
        ),
    )
}

#[deprecated]
pub fn parse_statements<T: BufRead + 'static, F, S: AsRef<str> + 'static>(
    lexer: &mut BufLexer<T>,
    exit_predicate: F,
    err_msg: S,
) -> Result<StatementNodes, QErrorNode>
where
    F: Fn(Option<&LexemeNode>) -> bool + 'static,
{
    take_if_statements(exit_predicate, err_msg)(lexer)
        .transpose()
        .map(|x| x.unwrap_or_default())
}

#[deprecated]
pub fn take_if_statements<T: BufRead + 'static, F, S: AsRef<str> + 'static>(
    exit_predicate: F,
    err_msg: S,
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNodes>>
where
    F: Fn(Option<&LexemeNode>) -> bool + 'static,
{
    take_if_statements_with_options(
        exit_predicate,
        ParseStatementsOptions {
            first_statement_separated_by_whitespace: false,
            err: QError::SyntaxError(err_msg.as_ref().to_string()),
        },
    )
}

#[deprecated]
pub fn parse_statements_with_options<T: BufRead + 'static, F>(
    lexer: &mut BufLexer<T>,
    exit_predicate: F,
    options: ParseStatementsOptions,
) -> Result<StatementNodes, QErrorNode>
where
    F: Fn(Option<&LexemeNode>) -> bool + 'static,
{
    take_if_statements_with_options(exit_predicate, options)(lexer)
        .transpose()
        .map(|x| x.unwrap_or_default())
}

#[deprecated]
pub fn take_if_statements_with_options<T: BufRead + 'static, F>(
    exit_predicate: F,
    options: ParseStatementsOptions,
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNodes>>
where
    F: Fn(Option<&LexemeNode>) -> bool + 'static,
{
    Box::new(move |lexer| {
        let mut read_separator = false;
        let mut statements: StatementNodes = vec![];
        loop {
            lexer.begin_transaction();
            let next: OptRes<LexemeNode> = lexer.next();
            match next {
                Some(Err(err)) => {
                    lexer.rollback_transaction();
                    return Some(Err(err));
                }
                None => {
                    if exit_predicate(None) {
                        // EOF is a valid exit predicate
                        lexer.rollback_transaction();
                        return Some(Ok(statements));
                    } else {
                        lexer.commit_transaction();
                        // TODO try to avoid clone here
                        return Some(Err(options.err.clone()).with_err_at(lexer.pos()));
                    }
                }
                Some(Ok(lexeme_node)) => {
                    if exit_predicate(Some(&lexeme_node)) {
                        lexer.rollback_transaction();
                        return Some(Ok(statements));
                    } else {
                        if lexeme_node.is_whitespace() {
                            if statements.is_empty()
                                && options.first_statement_separated_by_whitespace
                            {
                                read_separator = true;
                            }
                            lexer.commit_transaction();
                        } else if lexeme_node.is_eol() {
                            read_separator = true;
                            lexer.commit_transaction();
                        } else if lexeme_node.is_symbol('\'') {
                            // TODO roll back to allow comment to do its thing or expose a new method in comment
                            lexer.rollback_transaction();
                            // TODO do not do the double unwrap in case of IO error
                            let s = comment::take_if_comment()(lexer).unwrap().unwrap();
                            statements.push(s);
                        } else if lexeme_node.is_symbol(':') {
                            read_separator = true;
                            lexer.commit_transaction();
                        } else {
                            lexer.rollback_transaction();
                            if read_separator {
                                // TODO roll back or allow lexer to start with an existing lexeme
                                match statement::take_if_statement()(lexer) {
                                    Some(Ok(s)) => {
                                        statements.push(s);
                                        read_separator = false;
                                    }
                                    Some(Err(err)) => {
                                        return Some(Err(err));
                                    }
                                    None => {
                                        return Some(
                                            Err(QError::SyntaxError(format!("Expected statement")))
                                                .with_err_at(lexeme_node),
                                        );
                                    }
                                }
                            } else {
                                return Some(
                                    Err(QError::SyntaxError(format!(
                                        "Statement without separator: {:?}",
                                        lexeme_node
                                    )))
                                    .with_err_at(lexeme_node),
                                );
                            }
                        }
                    }
                }
            }
        }
    })
}
