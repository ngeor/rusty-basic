use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParseError, *};

pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    parse_get_or_put(Keyword::Get, BuiltInSub::Get)
}

pub fn parse_get_or_put(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq5(
        keyword(k),
        whitespace(),
        file_handle_p().or_syntax_error("Expected: file-number"),
        comma(),
        expression_pos_p().or_syntax_error("Expected: record-number"),
        move |_, _, file_number_pos, _, record_number_expr_pos| {
            Statement::built_in_sub_call(
                built_in_sub,
                vec![
                    file_number_pos.map(Expression::from),
                    record_number_expr_pos,
                ],
            )
        },
    )
}
