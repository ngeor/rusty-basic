use crate::expression::expression_node_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

// DEF SEG(=expr)?
pub fn parse() -> impl Parser<Output = Statement> {
    seq2(
        keyword_pair(Keyword::Def, Keyword::Seg),
        equal_sign_and_expression().allow_none(),
        |_, opt_expr_node| {
            Statement::BuiltInSubCall(BuiltInSub::DefSeg, opt_expr_node.into_iter().collect())
        },
    )
}

fn equal_sign_and_expression() -> impl Parser<Output = ExpressionNode> {
    equal_sign()
        .then_demand(expression_node_p().or_syntax_error("Expected expression after equal sign"))
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::*;

    #[test]
    fn parse_no_items_is_allowed() {
        let input = "DEF SEG";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::DefSeg, vec![])
        );
    }

    #[test]
    fn parse_one_item() {
        let input = "DEF SEG = 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::DefSeg, vec![42.as_lit_expr(1, 11)])
        );
    }
}
