use rusty_pc::*;

use crate::expr::{demand_ws_expr_ws_keyword_p, ws_expr_pos_p};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq3(
        keyword(Keyword::Name),
        demand_ws_expr_ws_keyword_p("old file name", Keyword::As),
        ws_expr_pos_p().or_expected("new file name"),
        |_, l, r| Statement::built_in_sub_call(BuiltInSub::Name, vec![l, r]),
    )
}
