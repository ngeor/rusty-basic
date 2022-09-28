use crate::parser::expression::guarded_expression_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements::*;
use crate::parser::types::*;

pub fn do_loop_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::Do)
        .then_use(do_condition_top().or(do_condition_bottom()))
        .map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<Output = DoLoopNode> {
    seq4(
        whitespace().and(keyword_choice(&[Keyword::Until, Keyword::While])),
        guarded_expression_node_p().or_syntax_error("Expected: expression"),
        ZeroOrMoreStatements::new(keyword(Keyword::Loop)),
        keyword(Keyword::Loop),
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

fn do_condition_bottom() -> impl NonOptParser<Output = DoLoopNode> {
    NonOptSeq5::new(
        ZeroOrMoreStatements::new(keyword(Keyword::Loop)),
        keyword(Keyword::Loop),
        whitespace(),
        keyword_choice(&[Keyword::Until, Keyword::While]),
        guarded_expression_node_p().or_syntax_error("Expected: expression"),
    )
    .map(|(statements, _, _, (k, _), condition)| DoLoopNode {
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
