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
        keyword_followed_by_whitespace_p(Keyword::Color)
            .and_opt(expression::expression_node_p().csv_allow_missing())
            .keep_right()
            .map(|opt_args| {
                Statement::BuiltInSubCall(BuiltInSub::Color, map_args(opt_args.unwrap_or_default()))
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
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            args.require_numeric_argument(i)?;
        }
        Ok(())
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
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn parse_foreground_only() {
        let input = "COLOR 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![1.as_lit_expr(1, 1), 7.as_lit_expr(1, 7)]
            )
        );
    }

    #[test]
    fn parse_background_only() {
        let input = "COLOR , 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![
                    0.as_lit_expr(1, 1),
                    1.as_lit_expr(1, 1),
                    7.as_lit_expr(1, 9)
                ]
            )
        );
    }

    #[test]
    fn parse_both_colors() {
        let input = "COLOR 7, 3";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![
                    1.as_lit_expr(1, 1),
                    7.as_lit_expr(1, 7),
                    1.as_lit_expr(1, 1),
                    3.as_lit_expr(1, 10)
                ]
            )
        );
    }

    #[test]
    fn parse_no_args() {
        let input = "COLOR";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: whitespace after COLOR"),
            1,
            6
        );
    }

    #[test]
    fn lint_wrong_foreground_type() {
        let input = "COLOR A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_wrong_background_type() {
        let input = "COLOR , A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
