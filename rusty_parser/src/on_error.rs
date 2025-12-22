use crate::expression::expression_pos_p;
use crate::name::bare_name_with_dots;
use crate::pc::*;
use crate::pc_specific::*;
use crate::{Expression, Keyword, OnErrorOption, ParseError, Statement};
use rusty_common::Positioned;

pub fn statement_on_error_go_to_p() -> impl Parser<RcStringView, Output = Statement> {
    Seq2::new(keyword_pair(Keyword::On, Keyword::Error), whitespace())
        .and_without_undo_keep_right(
            next()
                .or(goto())
                .or_syntax_error("Expected: GOTO or RESUME"),
        )
        .map(Statement::OnError)
}

fn next() -> impl Parser<RcStringView, Output = OnErrorOption> {
    // TODO implement a fn_map that ignores its input
    keyword_pair(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto() -> impl Parser<RcStringView, Output = OnErrorOption> {
    keyword_followed_by_whitespace_p(Keyword::GoTo).and_without_undo_keep_right(
        goto_label()
            .or(goto_zero())
            .or_syntax_error("Expected: label or 0"),
    )
}

fn goto_label() -> impl Parser<RcStringView, Output = OnErrorOption> {
    bare_name_with_dots().map(OnErrorOption::Label)
}

fn goto_zero() -> impl Parser<RcStringView, Output = OnErrorOption> {
    expression_pos_p().flat_map(|input, Positioned { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok((input, OnErrorOption::Zero)),
        _ => Err((
            true,
            input,
            ParseError::syntax_error("Expected: label or 0"),
        )),
    })
}
