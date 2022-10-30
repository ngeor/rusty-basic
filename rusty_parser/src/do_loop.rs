use crate::expression::ws_expr_node;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statements::*;
use crate::types::*;

pub fn do_loop_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::Do)
        .then_demand(do_condition_top().or(do_condition_bottom()))
        .map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<Output = DoLoopNode> {
    seq4(
        whitespace().and(keyword_choice(&[Keyword::Until, Keyword::While])),
        ws_expr_node().or_syntax_error("Expected: expression"),
        ZeroOrMoreStatements::new(keyword(Keyword::Loop)),
        keyword(Keyword::Loop).no_incomplete(),
        |(_, (k, _)), condition, statements, _| DoLoopNode {
            condition,
            statements,
            position: DoLoopConditionPosition::Top,
            kind: if k == Keyword::While {
                DoLoopConditionKind::While
            } else {
                DoLoopConditionKind::Until
            },
        },
    )
}

fn do_condition_bottom() -> impl Parser<Output = DoLoopNode> + NonOptParser {
    seq5_non_opt(
        ZeroOrMoreStatements::new(keyword(Keyword::Loop)),
        keyword(Keyword::Loop).no_incomplete(),
        whitespace().no_incomplete(),
        keyword_choice(&[Keyword::Until, Keyword::While]).no_incomplete(),
        ws_expr_node().or_syntax_error("Expected: expression"),
        |statements, _, _, (k, _), condition| DoLoopNode {
            condition,
            statements,
            position: DoLoopConditionPosition::Bottom,
            kind: if k == Keyword::While {
                DoLoopConditionKind::While
            } else {
                DoLoopConditionKind::Until
            },
        },
    )
}
