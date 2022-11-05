use crate::expression::csv_expressions_first_guarded;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse() -> impl Parser<Output = Statement> {
    seq2(
        keyword(Keyword::Data),
        csv_expressions_first_guarded().allow_default(),
        |_, args| Statement::BuiltInSubCall(BuiltInSub::Data, args),
    )
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{DemandSingleStatement, ExpressionLiteralFactory};
    use crate::*;

    #[test]
    fn parse_no_items_is_allowed() {
        let input = "DATA";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Data, vec![])
        );
    }

    #[test]
    fn parse_one_item() {
        let input = "DATA 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Data, vec![42.as_lit_expr(1, 6)])
        );
    }

    #[test]
    fn parse_two_items() {
        let input = r#"DATA 42, "hello""#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Data,
                vec![42.as_lit_expr(1, 6), "hello".as_lit_expr(1, 10)]
            )
        );
    }
}
