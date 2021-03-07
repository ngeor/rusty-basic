use crate::common::*;
use crate::parser::expression::guarded_expression_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::{keyword_choice_p, keyword_p, PcSpecific};
use crate::parser::statements::*;
use crate::parser::types::*;

pub fn do_loop_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Do)
        .and_demand(
            do_condition_top()
                .or(do_condition_bottom())
                .or_syntax_error("Expected: WHILE, UNTIL or statement after DO"),
        )
        .keep_right()
        .map(Statement::DoLoop)
}

fn do_condition_top<R>() -> impl Parser<R, Output = DoLoopNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(keyword_choice_p(&[Keyword::Until, Keyword::While]))
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

fn do_condition_bottom<R>() -> impl Parser<R, Output = DoLoopNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    zero_or_more_statements_p(keyword_p(Keyword::Loop))
        .and_demand(keyword_p(Keyword::Loop).or_syntax_error("DO without LOOP"))
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after LOOP"))
        .and_demand(
            keyword_choice_p(&[Keyword::Until, Keyword::While])
                .or_syntax_error("Expected: UNTIL or WHILE"),
        )
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
