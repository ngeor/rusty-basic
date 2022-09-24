use crate::parser::expression::guarded_expression_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements::*;
use crate::parser::types::*;

pub fn do_loop_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::Do)
        .and_demand(do_condition_top().or(do_condition_bottom()))
        .keep_right()
        .fn_map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<Output = DoLoopNode> {
    keyword_choice(&[Keyword::Until, Keyword::While])
        .preceded_by_req_ws()
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression"))
        .and_demand(ZeroOrMoreStatements::new(keyword(Keyword::Loop)))
        .and_demand(keyword(Keyword::Loop))
        .fn_map(|((((k, _), condition), statements), _)| DoLoopNode {
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

fn do_condition_bottom() -> impl NonOptParser<Output = DoLoopNode> {
    ZeroOrMoreStatements::new(keyword(Keyword::Loop))
        .and_demand(keyword(Keyword::Loop))
        .and_demand(keyword_choice(&[Keyword::Until, Keyword::While]).preceded_by_req_ws())
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression"))
        .fn_map(|(((statements, _), (k, _)), condition)| DoLoopNode {
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
