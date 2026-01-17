use rusty_pc::*;

use crate::core::statement::{single_line_non_comment_statement_p, single_line_statement_p};
use crate::input::RcStringView;
use crate::pc_specific::WithPos;
use crate::tokens::{colon_ws, whitespace_ignoring};
use crate::{ParserError, Statements};

pub fn single_line_non_comment_statements_p()
-> impl Parser<RcStringView, Output = Statements, Error = ParserError> {
    whitespace_ignoring().and_keep_right(delimited_by_colon(
        single_line_non_comment_statement_p().with_pos(),
    ))
}

pub fn single_line_statements_p()
-> impl Parser<RcStringView, Output = Statements, Error = ParserError> {
    whitespace_ignoring().and_keep_right(delimited_by_colon(single_line_statement_p().with_pos()))
}

fn delimited_by_colon<P: Parser<RcStringView, Error = ParserError>>(
    parser: P,
) -> impl Parser<RcStringView, Output = Vec<P::Output>, Error = ParserError> {
    parser.delimited_by(
        colon_ws(),
        ParserError::syntax_error("Error: trailing colon"),
    )
}
