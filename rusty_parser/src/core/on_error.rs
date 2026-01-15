use rusty_common::Positioned;
use rusty_pc::*;

use crate::core::expression::expression_pos_p;
use crate::core::name::bare_name_p;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{Expression, Keyword, OnErrorOption, Statement};

pub fn statement_on_error_go_to_p()
-> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq2(
        keyword_pair(Keyword::On, Keyword::Error),
        whitespace_ignoring(),
        |_, _| (),
    )
    .and_keep_right(next().or(goto()).or_expected("GOTO or RESUME"))
    .map(Statement::OnError)
}

fn next() -> impl Parser<RcStringView, Output = OnErrorOption, Error = ParseError> {
    // TODO implement a fn_map that ignores its input
    keyword_pair(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto() -> impl Parser<RcStringView, Output = OnErrorOption, Error = ParseError> {
    keyword_ws_p(Keyword::GoTo)
        .and_keep_right(goto_label().or(goto_zero()).or_expected("label or 0"))
}

fn goto_label() -> impl Parser<RcStringView, Output = OnErrorOption, Error = ParseError> {
    bare_name_p().map(OnErrorOption::Label)
}

fn goto_zero() -> impl Parser<RcStringView, Output = OnErrorOption, Error = ParseError> {
    expression_pos_p().flat_map(|input, Positioned { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok((input, OnErrorOption::Zero)),
        _ => Err((true, input, ParseError::expected("label or 0"))),
    })
}
