use rusty_pc::*;

use crate::core::statement::{single_line_non_comment_statement_p, single_line_statement_p};
use crate::input::RcStringView;
use crate::pc_specific::WithPos;
use crate::tokens::{colon_ws, whitespace};
use crate::{ParseError, Statements};

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
