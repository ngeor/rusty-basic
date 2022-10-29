use crate::parser::expression::{ws_expr_node, ws_expr_node_ws};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Statement> {
    keyword_pair(Keyword::View, Keyword::Print)
        .then_demand(parse_args().allow_default())
        .map(|opt_args| Statement::BuiltInSubCall(BuiltInSub::ViewPrint, opt_args))
}

fn parse_args() -> impl Parser<Output = ExpressionNodes> {
    seq3(
        ws_expr_node_ws(),
        keyword(Keyword::To).no_incomplete(),
        ws_expr_node().or_syntax_error("Expected: expression"),
        |l, _, r| vec![l, r],
    )
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::{BuiltInSub, Statement};

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
