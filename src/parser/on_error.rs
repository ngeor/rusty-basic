use crate::common::{Locatable, QError};
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::parsers::{FnMapTrait, KeepRightTrait, OrTrait, Parser};
use crate::parser::expression::expression_node_p;
use crate::parser::name::bare_name_p;
use crate::parser::specific::{
    keyword_followed_by_whitespace_p, keyword_pair_p, whitespace, OrSyntaxErrorTrait,
};
use crate::parser::{Expression, Keyword, OnErrorOption, Statement};

pub fn statement_on_error_go_to_p() -> impl Parser<Output = Statement> {
    keyword_pair_p(Keyword::On, Keyword::Error)
        .and_demand(whitespace())
        .and_demand(
            next()
                .or(goto())
                .or_syntax_error("Expected: GOTO or RESUME"),
        )
        .fn_map(|(_, r)| Statement::OnError(r))
}

fn next() -> impl Parser<Output = OnErrorOption> {
    // TODO implement a fn_map that ignores its input
    keyword_pair_p(Keyword::Resume, Keyword::Next).fn_map(|_| OnErrorOption::Next)
}

fn goto<R>() -> impl Parser<Output = OnErrorOption> {
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_demand(
            goto_label()
                .or(goto_zero())
                .or_syntax_error("Expected: label or 0"),
        )
        .keep_right()
}

fn goto_label() -> impl Parser<Output = OnErrorOption> {
    bare_name_p().fn_map(OnErrorOption::Label)
}

fn goto_zero() -> impl Parser<Output = OnErrorOption> {
    expression_node_p().and_then(|Locatable { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok(OnErrorOption::Zero),
        _ => Err(QError::syntax_error("Expected: label or 0")),
    })
}
