use rusty_pc::*;

use crate::core::statement::{
    single_line_non_comment_statement_p, single_line_statement_p, statement_p
};
use crate::core::statement_separator::{comment_separator, common_separator};
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::*;

pub fn single_line_non_comment_statements_p()
-> impl Parser<RcStringView, Output = Statements, Error = ParseError> {
    whitespace().and_keep_right(delimited_by_colon(
        single_line_non_comment_statement_p().with_pos(),
    ))
}

pub fn single_line_statements_p()
-> impl Parser<RcStringView, Output = Statements, Error = ParseError> {
    whitespace().and_keep_right(delimited_by_colon(single_line_statement_p().with_pos()))
}

fn delimited_by_colon<P: Parser<RcStringView, Error = ParseError>>(
    parser: P,
) -> impl Parser<RcStringView, Output = Vec<P::Output>, Error = ParseError> {
    delimited_by(
        parser,
        colon_ws(),
        ParseError::syntax_error("Error: trailing colon"),
    )
}

pub struct ZeroOrMoreStatements(Vec<Keyword>, Option<ParseError>);

impl ZeroOrMoreStatements {
    pub fn new(exit_source: Keyword) -> Self {
        Self(vec![exit_source], None)
    }

    pub fn new_multi(exit_source: Vec<Keyword>) -> Self {
        Self(exit_source, None)
    }

    pub fn new_with_custom_error(exit_source: Keyword, err: ParseError) -> Self {
        Self(vec![exit_source], Some(err))
    }

    fn found_exit(&self, tokenizer: RcStringView) -> ParseResult<RcStringView, bool, ParseError> {
        peek_token()
            .flat_map_ok_none(
                |input, token| {
                    for k in &self.0 {
                        if k == &token {
                            return Ok((input, true));
                        }
                    }
                    Ok((input, false))
                },
                |input| {
                    // EOF is an error here as we're looking for the exit source
                    match self.1.clone() {
                        Some(custom_err) => Err((true, input, custom_err)),
                        None => Err((
                            true,
                            input,
                            ParseError::SyntaxError(keyword_syntax_error(&self.0)),
                        )),
                    }
                },
            )
            .parse(tokenizer)
    }
}

impl Parser<RcStringView> for ZeroOrMoreStatements {
    type Output = Statements;
    type Error = ParseError;
    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        // must start with a separator (e.g. after a WHILE condition)
        let mut tokenizer = match common_separator().parse(tokenizer) {
            Ok((tokenizer, _)) => tokenizer,
            Err((false, i, _)) => {
                return Err((
                    true,
                    i,
                    ParseError::syntax_error("Expected: end-of-statement"),
                ));
            }
            Err(err) => {
                return Err(err);
            }
        };

        let mut result: Statements = vec![];
        // TODO rewrite the numeric state or add constants
        let mut state = 0;
        loop {
            // while not found exit
            let found_exit = match self.found_exit(tokenizer) {
                Ok((remaining, x)) => {
                    tokenizer = remaining;
                    x
                }
                Err((false, remaining, _)) => {
                    tokenizer = remaining;
                    false
                }
                Err(err) => return Err(err),
            };
            if found_exit {
                break;
            }

            if state == 0 || state == 2 {
                // looking for statement
                match statement_p().with_pos().parse(tokenizer) {
                    Ok((remaining, statement_pos)) => {
                        tokenizer = remaining;
                        result.push(statement_pos);
                        state = 1;
                    }
                    Err((false, remaining, _)) => {
                        return Err((
                            true,
                            remaining,
                            match &self.1 {
                                Some(custom_error) => custom_error.clone(),
                                _ => ParseError::syntax_error("Expected: statement"),
                            },
                        ));
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else if state == 1 {
                // looking for separator after statement
                let found_separator =
                    if let Some(Statement::Comment(_)) = result.last().map(|x| &x.element) {
                        // last element was comment
                        match comment_separator().parse(tokenizer) {
                            Ok((remaining, _)) => {
                                tokenizer = remaining;
                                true
                            }
                            Err((false, remaining, _)) => {
                                tokenizer = remaining;
                                false
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    } else {
                        match common_separator().parse(tokenizer) {
                            Ok((remaining, _)) => {
                                tokenizer = remaining;
                                true
                            }
                            Err((false, remaining, _)) => {
                                tokenizer = remaining;
                                false
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    };
                if found_separator {
                    state = 2;
                } else {
                    return Err((
                        true,
                        tokenizer,
                        ParseError::syntax_error("Expected: statement separator"),
                    ));
                }
            } else {
                panic!("Cannot happen")
            }
        }
        Ok((tokenizer, result))
    }
}
