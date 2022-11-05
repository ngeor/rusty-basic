use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorPos, WithErrNoPos};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
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
    use rusty_common::*;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LEFT$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_one_arg() {
        assert_linter_err!(r#"PRINT LEFT$("oops")"#, QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_three_args() {
        assert_linter_err!(
            r#"PRINT LEFT$("oops", 1, 2)"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LEFT$(10, 40)"#, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn test_second_arg_string() {
        assert_linter_err!(
            r#"PRINT LEFT$("hello", "world")"#,
            QError::ArgumentTypeMismatch
        );
    }
}
