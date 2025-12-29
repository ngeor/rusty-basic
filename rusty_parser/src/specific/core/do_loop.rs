use crate::input::RcStringView;
use crate::specific::core::expression::ws_expr_pos_p;
use crate::specific::core::statements::ZeroOrMoreStatements;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::{pc::*, ParseError};

pub fn do_loop_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    keyword(Keyword::Do)
        .and_keep_right(
            do_condition_top()
                .or(do_condition_bottom())
                .or_syntax_error("Syntax error in DO loop"),
        )
        .map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<RcStringView, Output = DoLoop, Error = ParseError> {
    seq4(
        whitespace().and_tuple(keyword_choice(vec![Keyword::Until, Keyword::While])),
        ws_expr_pos_p().or_syntax_error("Expected: expression"),
        ZeroOrMoreStatements::new(Keyword::Loop),
        keyword(Keyword::Loop),
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

fn do_condition_bottom() -> impl Parser<RcStringView, Output = DoLoop, Error = ParseError> {
    seq5(
        ZeroOrMoreStatements::new(Keyword::Loop),
        keyword(Keyword::Loop),
        whitespace(),
        keyword_choice(vec![Keyword::Until, Keyword::While]),
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
