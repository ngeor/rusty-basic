use rusty_pc::*;

use crate::expr::expression_pos_p;
use crate::expr::file_handle::file_handle_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    parse_get_or_put(Keyword::Get, BuiltInSub::Get)
}

pub fn parse_get_or_put(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword_ws_p(k),
        file_handle_p().or_expected("file-number"),
        comma_ws(),
        expression_pos_p().or_expected("record-number"),
        move |_, file_number_pos, _, record_number_expr_pos| {
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
