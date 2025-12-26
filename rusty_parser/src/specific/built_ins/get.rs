use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::BuiltInSub;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    parse_get_or_put(Keyword::Get, BuiltInSub::Get)
}

pub fn parse_get_or_put(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<RcStringView, Output = Statement> {
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
