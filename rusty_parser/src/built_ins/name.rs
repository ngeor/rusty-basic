use crate::expression::{ws_expr_node, ws_expr_node_ws};
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse() -> impl Parser<Output = Statement> {
    seq4(
        keyword(Keyword::Name),
        ws_expr_node_ws().or_syntax_error("Expected: old file name"),
        keyword(Keyword::As).no_incomplete(),
        ws_expr_node().or_syntax_error("Expected: new file name"),
        |_, l, _, r| Statement::BuiltInSubCall(BuiltInSub::Name, vec![l, r]),
    )
}
