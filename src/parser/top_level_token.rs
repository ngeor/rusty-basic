use crate::common::*;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::implementation;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc2::Parser;
use crate::parser::pc_specific::with_pos;
use crate::parser::statement;
use crate::parser::types::*;
use crate::parser::user_defined_type;

pub fn top_level_tokens<R>() -> Box<dyn Fn(R) -> ReaderResult<R, ProgramNode, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
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

pub fn top_level_token_one<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelTokenNode, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    with_pos(or_vec(vec![
        top_level_token_def_type(),
        top_level_token_declaration(),
        top_level_token_implementation(),
        top_level_token_statement(),
        top_level_token_user_defined_type(),
    ]))
}

fn top_level_token_def_type<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(def_type::def_type_p().convert_to_fn(), |d| {
        TopLevelToken::DefType(d)
    })
}

fn top_level_token_declaration<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    declaration::declaration_p().convert_to_fn()
}

fn top_level_token_implementation<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    implementation::implementation()
}

fn top_level_token_statement<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(statement::statement(), |s| TopLevelToken::Statement(s))
}

fn top_level_token_user_defined_type<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(user_defined_type::user_defined_type(), |u| {
        TopLevelToken::UserDefinedType(u)
    })
}
