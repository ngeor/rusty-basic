use crate::expression::expression_pos_p;
use crate::name::bare_name_with_dots;
use crate::pc::*;
use crate::pc_specific::*;
use crate::{Expression, Keyword, OnErrorOption, ParseError, Statement};
use rusty_common::Positioned;

pub fn statement_on_error_go_to_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    Seq2::new(
        keyword_pair(Keyword::On, Keyword::Error),
        whitespace().no_incomplete(),
    )
    .and_without_undo_keep_right(
        next()
            .or(goto())
            .or_syntax_error("Expected: GOTO or RESUME"),
    )
    .map(Statement::OnError)
}

fn next<I: Tokenizer + 'static>() -> impl Parser<I, Output = OnErrorOption> {
    // TODO implement a fn_map that ignores its input
    keyword_pair(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto<I: Tokenizer + 'static>() -> impl Parser<I, Output = OnErrorOption> {
    keyword_followed_by_whitespace_p(Keyword::GoTo).and_without_undo_keep_right(
        goto_label()
            .or(goto_zero())
            .or_syntax_error("Expected: label or 0"),
    )
}

fn goto_label<I: Tokenizer + 'static>() -> impl Parser<I, Output = OnErrorOption> {
    bare_name_with_dots().map(OnErrorOption::Label)
}

fn goto_zero<I: Tokenizer + 'static>() -> impl Parser<I, Output = OnErrorOption> {
    expression_pos_p().flat_map(|Positioned { element, .. }| match element {
        Expression::IntegerLiteral(0) => ParseResult::Ok(OnErrorOption::Zero),
        _ => ParseResult::Err(ParseError::syntax_error("Expected: label or 0")),
    })
}
