use crate::parser::base::parsers::{AndDemandTrait, AndTrait, KeepRightTrait, Parser};
use crate::parser::expression::guarded_expression_node_p;
use crate::parser::specific::{keyword, keyword_choice, keyword_p, whitespace};
use crate::parser::statements::*;
use crate::parser::types::*;

pub fn do_loop_p() -> impl Parser<Output = Statement> {
    keyword_p(Keyword::Do)
        .and_demand(
            do_condition_top()
                .or(do_condition_bottom())
                .or_syntax_error("Expected: WHILE, UNTIL or statement after DO"),
        )
        .keep_right()
        .map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<Output = DoLoopNode> {
    whitespace()
        .and(keyword_choice(&[Keyword::Until, Keyword::While]))
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression"))
        .and_demand(zero_or_more_statements_p(keyword_p(Keyword::Loop)))
        .and_demand(keyword_p(Keyword::Loop).or_syntax_error("DO without LOOP"))
        .map(|((((_, (k, _)), condition), statements), _)| DoLoopNode {
            condition,
            statements,
            position: DoLoopConditionPosition::Top,
            kind: if k == Keyword::While {
                DoLoopConditionKind::While
            } else {
                DoLoopConditionKind::Until
            },
        })
}

fn do_condition_bottom() -> impl Parser<Output = DoLoopNode> {
    zero_or_more_statements_p(keyword_p(Keyword::Loop))
        .and_demand(keyword(Keyword::Loop))
        .and_demand(whitespace())
        .and_demand(keyword_choice(&[Keyword::Until, Keyword::While]))
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression"))
        .map(|((((statements, _), _), (k, _)), condition)| DoLoopNode {
            condition,
            statements,
            position: DoLoopConditionPosition::Bottom,
            kind: if k == Keyword::While {
                DoLoopConditionKind::While
            } else {
                DoLoopConditionKind::Until
            },
        })
}
