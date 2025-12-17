use crate::pc::*;
use crate::pc_specific::*;
use crate::statement_separator::{comment_separator, common_separator};
use crate::types::*;
use crate::{statement, ParseError};

pub fn single_line_non_comment_statements_p<I: Tokenizer + 'static>(
) -> impl Parser<I, Output = Statements> {
    whitespace()
        .and(delimited_by_colon(
            statement::single_line_non_comment_statement_p().with_pos(),
        ))
        .keep_right()
}

pub fn single_line_statements_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statements> {
    whitespace()
        .and(delimited_by_colon(
            statement::single_line_statement_p().with_pos(),
        ))
        .keep_right()
}

fn delimited_by_colon<I: Tokenizer + 'static, P: Parser<I>>(
    parser: P,
) -> impl Parser<I, Output = Vec<P::Output>> {
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

    fn found_exit<I: Tokenizer + 'static>(
        &self,
        tokenizer: &mut I,
    ) -> ParseResult<bool, ParseError> {
        peek_token()
            .and_then_ok_err(
                |token| {
                    for k in &self.0 {
                        if k == &token {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
                |_| {
                    // EOF is an error here as we're looking for the exit source
                    match self.1.clone() {
                        Some(custom_err) => Err(custom_err),
                        None => Err(ParseError::SyntaxError(keyword_syntax_error(&self.0))),
                    }
                },
            )
            .parse(tokenizer)
    }
}

impl<I: Tokenizer + 'static> Parser<I> for ZeroOrMoreStatements {
    type Output = Statements;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        // must start with a separator (e.g. after a WHILE condition)
        match common_separator().parse_opt(tokenizer) {
            ParseResult::Ok(Some(_)) => { /*ok*/ }
            ParseResult::Ok(None) => {
                return ParseResult::Err(ParseError::syntax_error("Expected: end-of-statement"));
            }
            ParseResult::Err(err) => {
                return ParseResult::Err(err);
            }
        };

        let mut result: Statements = vec![];
        // TODO rewrite the numeric state or add constants
        let mut state = 0;
        loop {
            // while not found exit
            let found_exit = match self.found_exit(tokenizer) {
                ParseResult::Ok(x) => x,
                ParseResult::Err(err) => return ParseResult::Err(err),
            };
            if found_exit {
                break;
            }

            if state == 0 || state == 2 {
                // looking for statement
                match statement::statement_p().with_pos().parse_opt(tokenizer) {
                    ParseResult::Ok(Some(statement_pos)) => {
                        result.push(statement_pos);
                        state = 1;
                    }
                    ParseResult::Ok(None) => {
                        return ParseResult::Err(match &self.1 {
                            Some(custom_error) => custom_error.clone(),
                            _ => ParseError::syntax_error("Expected: statement"),
                        });
                    }
                    ParseResult::Err(err) => {
                        return ParseResult::Err(err);
                    }
                }
            } else if state == 1 {
                // looking for separator after statement
                let found_separator =
                    if let Some(Statement::Comment(_)) = result.last().map(|x| &x.element) {
                        // last element was comment
                        match comment_separator().parse_opt(tokenizer) {
                            ParseResult::Ok(opt) => opt.is_some(),
                            ParseResult::Err(err) => {
                                return ParseResult::Err(err);
                            }
                        }
                    } else {
                        match common_separator().parse_opt(tokenizer) {
                            ParseResult::Ok(opt) => opt.is_some(),
                            ParseResult::Err(err) => {
                                return ParseResult::Err(err);
                            }
                        }
                    };
                if found_separator {
                    state = 2;
                } else {
                    return ParseResult::Err(ParseError::syntax_error(
                        "Expected: statement separator",
                    ));
                }
            } else {
                panic!("Cannot happen")
            }
        }
        ParseResult::Ok(result)
    }
}
