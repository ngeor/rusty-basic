use crate::expression::{ws_expr_pos_p, ws_expr_pos_ws_p};
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    keyword_pair(Keyword::View, Keyword::Print)
        .then_demand(parse_args().or_default())
        .map(|opt_args| Statement::BuiltInSubCall(BuiltInSub::ViewPrint, opt_args))
}

fn parse_args<I: Tokenizer + 'static>() -> impl Parser<I, Output = Expressions> {
    seq3(
        ws_expr_pos_ws_p(),
        keyword(Keyword::To).no_incomplete(),
        ws_expr_pos_p().or_syntax_error("Expected: expression"),
        |l, _, r| vec![l, r],
    )
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{DemandSingleStatement, ExpressionLiteralFactory};
    use crate::{parse, BuiltInSub, Statement};

    #[test]
    fn parse_no_args() {
        let input = "VIEW PRINT";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::ViewPrint, vec![])
        );
    }

    #[test]
    fn parse_args() {
        let input = "VIEW PRINT 1 TO 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::ViewPrint,
                vec![1.as_lit_expr(1, 12), 20.as_lit_expr(1, 17)]
            )
        );
    }
}
