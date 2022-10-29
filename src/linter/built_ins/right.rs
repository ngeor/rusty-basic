use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)
    } else {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT RIGHT$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_one_arg() {
        assert_linter_err!(r#"PRINT RIGHT$("oops")"#, QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_three_args() {
        assert_linter_err!(
            r#"PRINT RIGHT$("oops", 1, 2)"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT RIGHT$(10, 40)"#, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn test_second_arg_string() {
        assert_linter_err!(
            r#"PRINT RIGHT$("hello", "world")"#,
            QError::ArgumentTypeMismatch
        );
    }
}
