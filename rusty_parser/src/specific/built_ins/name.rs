use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::BuiltInSub;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    seq4(
        keyword(Keyword::Name),
        ws_expr_pos_ws_p().or_syntax_error("Expected: old file name"),
        keyword(Keyword::As),
        ws_expr_pos_p().or_syntax_error("Expected: new file name"),
        |_, l, _, r| Statement::BuiltInSubCall(BuiltInSub::Name, vec![l, r]),
    )
}
