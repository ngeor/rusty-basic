use crate::expression::ws_expr_pos_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statements::*;
use crate::types::*;

pub fn do_loop_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    keyword(Keyword::Do)
        .then_demand(do_condition_top().or(do_condition_bottom()))
        .map(Statement::DoLoop)
}

fn do_condition_top<I: Tokenizer + 'static>() -> impl Parser<I, Output = DoLoop> {
    seq4(
        whitespace().and(keyword_choice(vec![Keyword::Until, Keyword::While])),
        ws_expr_pos_p().or_syntax_error("Expected: expression"),
        ZeroOrMoreStatements::new(Keyword::Loop),
        keyword(Keyword::Loop).no_incomplete(),
        |(_, (k, _)), condition, statements, _| DoLoop {
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

fn do_condition_bottom<I: Tokenizer + 'static>() -> impl Parser<I, Output = DoLoop> {
    seq5_non_opt(
        ZeroOrMoreStatements::new(Keyword::Loop),
        keyword(Keyword::Loop).no_incomplete(),
        whitespace().no_incomplete(),
        keyword_choice(vec![Keyword::Until, Keyword::While]).no_incomplete(),
        ws_expr_pos_p().or_syntax_error("Expected: expression"),
        |statements, _, _, (k, _), condition| DoLoop {
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
