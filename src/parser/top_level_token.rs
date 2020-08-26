// top level token ::=
//      comment |
//      def type |
//      declaration |
//      statement |
//      function implementation |
//      sub implementation |
//      whitespace - empty line

use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::implementation;
use crate::parser::pc::loc::*;
use crate::parser::pc::traits::*;
use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

pub fn top_level_tokens<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ProgramNode, QErrorNode>)> {
    Box::new(move |r| {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        let mut reader = r;
        loop {
            let (tmp, next) = reader.read();
            reader = tmp;
            match next {
                Ok(' ') => {
                    // skip whitespace
                }
                Ok('\r') | Ok('\n') | Ok(':') => {
                    read_separator = true;
                }
                Err(err) => {
                    if err.is_not_found_err() {
                        break;
                    } else {
                        return (reader, Err(err));
                    }
                }
                Ok(ch) => {
                    // if it is a comment, we are allowed to read it without a separator
                    let can_read = ch == '\'' || read_separator;
                    if can_read {
                        let (tmp, next) = top_level_token_one()(reader.undo(ch));
                        reader = tmp;
                        read_separator = false;
                        match next {
                            Ok(top_level_token) => {
                                top_level_tokens.push(top_level_token);
                            }
                            Err(err) => {
                                if err.is_not_found_err() {
                                    return (
                                        reader,
                                        Err(err.map(|_| {
                                            QError::SyntaxError(format!(
                                                "Expected top level statement"
                                            ))
                                        })),
                                    );
                                } else {
                                    return (reader, Err(err));
                                }
                            }
                        }
                    } else {
                        return wrap_err(
                            reader,
                            QError::SyntaxError(format!("No separator: {}", ch)),
                        );
                    }
                }
            }
        }

        (reader, Ok(top_level_tokens))
    })
}

pub fn top_level_token_one<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelTokenNode, QErrorNode>)> {
    with_pos(or_vec(vec![
        top_level_token_def_type(),
        top_level_token_declaration(),
        top_level_token_implementation(),
        top_level_token_statement(),
    ]))
}

pub fn top_level_token_def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map(def_type::def_type(), |d| TopLevelToken::DefType(d))
}

pub fn top_level_token_declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    declaration::declaration()
}

pub fn top_level_token_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    implementation::implementation()
}

pub fn top_level_token_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map(statement::statement(), |s| TopLevelToken::Statement(s))
}
