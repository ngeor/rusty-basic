use crate::common::{Locatable, QError};
use crate::parser::expression::expression_node_p;
use crate::parser::name::bare_name_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{Expression, Keyword, OnErrorOption, Statement};

pub fn statement_on_error_go_to_p() -> impl Parser<Output = Statement> {
    Seq2::new(
        keyword_pair(Keyword::On, Keyword::Error),
        whitespace().no_incomplete(),
    )
    .then_demand(
        next()
            .or(goto())
            .or_syntax_error("Expected: GOTO or RESUME"),
    )
    .map(Statement::OnError)
}

fn next() -> impl Parser<Output = OnErrorOption> {
    // TODO implement a fn_map that ignores its input
    keyword_pair(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto() -> impl Parser<Output = OnErrorOption> {
    keyword_followed_by_whitespace_p(Keyword::GoTo).then_demand(
        goto_label()
            .or(goto_zero())
            .or_syntax_error("Expected: label or 0"),
    )
}

fn goto_label() -> impl Parser<Output = OnErrorOption> {
    bare_name_p().map(OnErrorOption::Label)
}

fn goto_zero() -> impl Parser<Output = OnErrorOption> {
    expression_node_p().and_then(|Locatable { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok(OnErrorOption::Zero),
        _ => Err(QError::syntax_error("Expected: label or 0")),
    })
}
