use rusty_pc::*;

use crate::core::statement::{single_line_non_comment_statement_p, single_line_statement_p};
use crate::input::StringView;
use crate::pc_specific::{WithPos, lead_ws};
use crate::tokens::colon_ws;
use crate::{ParserError, Statements};

pub fn single_line_non_comment_statements_p()
-> impl Parser<StringView, Output = Statements, Error = ParserError> {
    lead_ws(delimited_by_colon(
        single_line_non_comment_statement_p().with_pos(),
    ))
}

pub fn single_line_statements_p()
-> impl Parser<StringView, Output = Statements, Error = ParserError> {
    lead_ws(delimited_by_colon(single_line_statement_p().with_pos()))
}

fn delimited_by_colon<P: Parser<StringView, Error = ParserError>>(
    parser: P,
) -> impl Parser<StringView, Output = Vec<P::Output>, Error = ParserError> {
    parser.delimited_by(
        colon_ws(),
        ParserError::syntax_error("Error: trailing colon"),
    )
}
