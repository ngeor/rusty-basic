use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParseError, *};

pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq4(
        keyword(Keyword::Name),
        ws_expr_pos_ws_p().or_expected("old file name"),
        keyword(Keyword::As),
        ws_expr_pos_p().or_expected("new file name"),
        |_, l, _, r| Statement::built_in_sub_call(BuiltInSub::Name, vec![l, r]),
    )
}
