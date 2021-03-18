pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::Locate)
            .and_opt(expression::expression_node_p().csv_allow_missing())
            .keep_right()
            .map(|opt_args| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Locate,
                    map_args(opt_args.unwrap_or_default()),
                )
            })
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
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // 2 or 3 arguments (row, col, cursor) but they're duplicated because they're optional
        if args.len() != 4 && args.len() != 6 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            for i in 0..args.len() {
                args.require_integer_argument(i)?;
            }
            Ok(())
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::Statement;

    #[test]
    fn parse_row_col() {
        let input = "LOCATE 10, 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    1.as_lit_expr(1, 1),   // row present
                    10.as_lit_expr(1, 8),  // row
                    1.as_lit_expr(1, 1),   // col present
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
                    0.as_lit_expr(1, 1),  // row absent
                    0.as_lit_expr(1, 1),  // col absent
                    1.as_lit_expr(1, 1),  // cursor present
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
