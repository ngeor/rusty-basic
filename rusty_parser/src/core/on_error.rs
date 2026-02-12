use rusty_common::Positioned;
use rusty_pc::*;

use crate::core::expression::expression_pos_p;
use crate::core::name::bare_name_p;
use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{Expression, Keyword, OnErrorOption, Statement};

pub fn statement_on_error_go_to_p()
-> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq2(
        keyword_pair(Keyword::On, Keyword::Error),
        whitespace_ignoring(),
        |_, _| (),
    )
    .and_keep_right(next().or(goto()).or_expected("GOTO or RESUME"))
    .map(Statement::OnError)
}

fn next() -> impl Parser<StringView, Output = OnErrorOption, Error = ParserError> {
    // TODO implement a fn_map that ignores its input
    keyword_pair(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto() -> impl Parser<StringView, Output = OnErrorOption, Error = ParserError> {
    keyword_ws_p(Keyword::GoTo)
        .and_keep_right(goto_label().or(goto_zero()).or_expected("label or 0"))
}

fn goto_label() -> impl Parser<StringView, Output = OnErrorOption, Error = ParserError> {
    bare_name_p().map(OnErrorOption::Label)
}

fn goto_zero() -> impl Parser<StringView, Output = OnErrorOption, Error = ParserError> {
    expression_pos_p().and_then(|Positioned { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok(OnErrorOption::Zero),
        _ => Err(ParserError::expected("label or 0").to_fatal()),
    })
}
