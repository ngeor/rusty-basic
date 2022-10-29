#[cfg(test)]
mod tests {
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn parse_without_args() {
        let input = "CLS";
        let statement = parse(input).demand_single_statement();
        assert_eq!(statement, Statement::SubCall("CLS".into(), vec![]));
    }

    #[test]
    fn parse_with_one_arg() {
        let input = "CLS 2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::SubCall("CLS".into(), vec![2.as_lit_expr(1, 5)])
        );
    }
}
