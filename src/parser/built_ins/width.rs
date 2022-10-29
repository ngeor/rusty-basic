use crate::common::*;
use crate::parser::built_ins::csv_allow_missing;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::Width)
        .then_demand(csv_allow_missing())
        .map(|opt_args| Statement::BuiltInSubCall(BuiltInSub::Width, map_args(opt_args)))
}

fn map_args(args: Vec<Option<ExpressionNode>>) -> ExpressionNodes {
    args.into_iter().flat_map(map_arg).collect()
}

fn map_arg(arg: Option<ExpressionNode>) -> ExpressionNodes {
    match arg {
        Some(a) => vec![Expression::IntegerLiteral(1).at(Location::start()), a],
        _ => vec![Expression::IntegerLiteral(0).at(Location::start())],
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::QError;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::{BuiltInSub, Statement};

    #[test]
    fn parse_row_col() {
        let input = "WIDTH 80, 24";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Width,
                vec![
                    1.as_lit_expr(1, 1),   // row present
                    80.as_lit_expr(1, 7),  // row
                    1.as_lit_expr(1, 1),   // col present
                    24.as_lit_expr(1, 11)  // col
                ]
            )
        );
    }

    #[test]
    fn parse_only_col_arg() {
        let input = "WIDTH , 24";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Width,
                vec![
                    0.as_lit_expr(1, 1),  // row absent
                    1.as_lit_expr(1, 1),  // col present
                    24.as_lit_expr(1, 9)  // col
                ]
            )
        );
    }

    #[test]
    fn cannot_have_trailing_comma() {
        assert_parser_err!("WIDTH 1, 2,", QError::syntax_error("Error: trailing comma"));
    }
}
