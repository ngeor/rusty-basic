use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::specific::*;
use crate::{BuiltInSub, ParseError};

pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    keyword_pair(Keyword::View, Keyword::Print)
        .and_keep_right(parse_args().or_default())
        .map(|opt_args| Statement::built_in_sub_call(BuiltInSub::ViewPrint, opt_args))
}

fn parse_args() -> impl Parser<RcStringView, Output = Expressions, Error = ParseError> {
    seq3(
        ws_expr_pos_ws_p(),
        keyword(Keyword::To),
        ws_expr_pos_p().or_syntax_error("Expected: expression"),
        |l, _, r| vec![l, r],
    )
}

#[cfg(test)]
mod tests {
    use crate::specific::*;
    use crate::test_utils::{DemandSingleStatement, ExpressionLiteralFactory};
    use crate::{BuiltInSub, parse};
    #[test]
    fn parse_no_args() {
        let input = "VIEW PRINT";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::built_in_sub_call(BuiltInSub::ViewPrint, vec![])
        );
    }

    #[test]
    fn parse_args() {
        let input = "VIEW PRINT 1 TO 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::built_in_sub_call(
                BuiltInSub::ViewPrint,
                vec![1.as_lit_expr(1, 12), 20.as_lit_expr(1, 17)]
            )
        );
    }
}
