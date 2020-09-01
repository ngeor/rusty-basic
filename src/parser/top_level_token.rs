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
use crate::parser::pc::common::*;
use crate::parser::pc::loc::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

pub fn top_level_tokens<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ProgramNode, QError>> {
    Box::new(move |r| {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        let mut reader = r;
        loop {
            match reader.read() {
                Ok((tmp, opt_res)) => {
                    reader = tmp;
                    match opt_res {
                        Some(' ') => {
                            // skip whitespace
                        }
                        Some('\r') | Some('\n') | Some(':') => {
                            read_separator = true;
                        }
                        Some(ch) => {
                            // if it is a comment, we are allowed to read it without a separator
                            let can_read = ch == '\'' || read_separator;
                            if can_read {
                                match top_level_token_one()(reader.undo(ch)) {
                                    Ok((tmp, opt_res)) => {
                                        reader = tmp;
                                        read_separator = false;
                                        match opt_res {
                                            Some(top_level_token) => {
                                                top_level_tokens.push(top_level_token);
                                            }
                                            None => {
                                                return Err((
                                                    reader,
                                                    QError::SyntaxError(format!(
                                                        "Expected: top level statement"
                                                    )),
                                                ));
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        return Err(err);
                                    }
                                }
                            } else {
                                return Err((
                                    reader,
                                    QError::SyntaxError(format!("No separator: {}", ch)),
                                ));
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok((reader, Some(top_level_tokens)))
    })
}

pub fn top_level_token_one<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelTokenNode, QError>> {
    with_pos(or_vec(vec![
        top_level_token_def_type(),
        top_level_token_declaration(),
        top_level_token_implementation(),
        top_level_token_statement(),
    ]))
}

pub fn top_level_token_def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(def_type::def_type(), |d| TopLevelToken::DefType(d))
}

pub fn top_level_token_declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    declaration::declaration()
}

pub fn top_level_token_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    implementation::implementation()
}

pub fn top_level_token_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(statement::statement(), |s| TopLevelToken::Statement(s))
}
