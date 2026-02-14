use rusty_pc::and::VecCombiner;
use rusty_pc::*;

use crate::expr::{ws_expr_pos_p, ws_expr_ws_keyword_p};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword_pair(Keyword::View, Keyword::Print)
        .and_keep_right(parse_args().or_default())
        .map(|opt_args| Statement::built_in_sub_call(BuiltInSub::ViewPrint, opt_args))
}

fn parse_args() -> impl Parser<StringView, Output = Expressions, Error = ParserError> {
    ws_expr_ws_keyword_p(Keyword::To).and(ws_expr_pos_p().or_expected("expression"), VecCombiner)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{DemandSingleStatement, ExpressionLiteralFactory};
    use crate::{BuiltInSub, parse, *};
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
