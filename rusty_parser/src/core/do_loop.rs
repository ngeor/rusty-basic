use rusty_pc::*;

use crate::core::expression::ws_expr_pos_p;
use crate::core::statements::zero_or_more_statements;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{ParserError, *};

pub fn do_loop_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword(Keyword::Do)
        .and_keep_right(
            do_condition_top()
                .or(do_condition_bottom())
                .or_fail(ParserError::syntax_error("Syntax error in DO loop")),
        )
        .map(Statement::DoLoop)
}

fn do_condition_top() -> impl Parser<StringView, Output = DoLoop, Error = ParserError> {
    seq4(
        lead_ws(keyword_of!(Keyword::Until, Keyword::While)),
        ws_expr_pos_p().or_expected("expression"),
        zero_or_more_statements!(Keyword::Loop),
        keyword(Keyword::Loop),
        |k, condition, statements, _| DoLoop {
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

fn do_condition_bottom() -> impl Parser<StringView, Output = DoLoop, Error = ParserError> {
    seq4(
        zero_or_more_statements!(Keyword::Loop),
        keyword_ws_p(Keyword::Loop),
        keyword_of!(Keyword::Until, Keyword::While),
        ws_expr_pos_p().or_expected("expression"),
        |statements, _, k, condition| DoLoop {
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
