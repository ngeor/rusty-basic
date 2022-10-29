use crate::parser::built_ins::parse_built_in_sub_with_opt_args;
use crate::parser::pc::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Statement> {
    // TODO limit to 2 args here so linter can be removed
    parse_built_in_sub_with_opt_args(Keyword::Locate, BuiltInSub::Locate)
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::{BuiltInSub, Statement};
    use rusty_common::*;

    #[test]
    fn parse_row() {
        let input = "LOCATE 11";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    1.as_lit_expr(1, 1),  // row present
                    11.as_lit_expr(1, 8), // row
                ]
            )
        );
    }

    #[test]
    fn parse_col() {
        let input = "LOCATE , 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    2.as_lit_expr(1, 1),   // col present
                    20.as_lit_expr(1, 10), // col
                ]
            )
        );
    }

    #[test]
    fn parse_row_col() {
        let input = "LOCATE 10, 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    3.as_lit_expr(1, 1),   // row and col present
                    10.as_lit_expr(1, 8),  // row
                    20.as_lit_expr(1, 12)  // col
                ]
            )
        );
    }

    #[test]
    fn parse_only_cursor_arg() {
        let input = "LOCATE , , 1";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    4.as_lit_expr(1, 1),  // cursor present
                    1.as_lit_expr(1, 12)  // cursor
                ]
            )
        );
    }

    #[test]
    fn cannot_have_trailing_comma() {
        assert_parser_err!(
            "LOCATE 1, 2,",
            QError::syntax_error("Error: trailing comma")
        );
    }
}
