use rusty_pc::*;

use crate::expr::{ws_expr_pos_p, ws_expr_pos_ws_p};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword(Keyword::Name),
        ws_expr_pos_ws_p().or_expected("old file name"),
        keyword(Keyword::As),
        ws_expr_pos_p().or_expected("new file name"),
        |_, l, _, r| Statement::built_in_sub_call(BuiltInSub::Name, vec![l, r]),
    )
}
