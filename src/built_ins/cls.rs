pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() {
            Ok(())
        } else if args.len() == 1 {
            args.require_numeric_argument(0)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        interpreter.screen().cls()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;
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

    #[test]
    fn lint_arg_wrong_type() {
        let input = "CLS A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_two_args() {
        let input = "CLS 42, 1";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }
}
