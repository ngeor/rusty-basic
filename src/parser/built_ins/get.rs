use crate::parser::expression::expression_node_p;
use crate::parser::expression::file_handle::file_handle_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Statement> {
    parse_get_or_put(Keyword::Get, BuiltInSub::Get)
}

pub fn parse_get_or_put(k: Keyword, built_in_sub: BuiltInSub) -> impl Parser<Output = Statement> {
    seq5(
        keyword(k),
        whitespace().no_incomplete(),
        file_handle_p().or_syntax_error("Expected: file-number"),
        comma().no_incomplete(),
        expression_node_p().or_syntax_error("Expected: record-number"),
        move |_, _, file_number_node, _, record_number_expr_node| {
            Statement::BuiltInSubCall(
                built_in_sub,
                vec![
                    file_number_node.map(Expression::from),
                    record_number_expr_node,
                ],
            )
        },
    )
}
