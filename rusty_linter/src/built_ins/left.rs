use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)
    } else {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LEFT$()"#, LintError::FunctionNeedsArguments);
    }

    #[test]
    fn test_one_arg() {
        assert_linter_err!(r#"PRINT LEFT$("oops")"#, LintError::ArgumentCountMismatch);
    }

    #[test]
    fn test_three_args() {
        assert_linter_err!(
            r#"PRINT LEFT$("oops", 1, 2)"#,
            LintError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LEFT$(10, 40)"#, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn test_second_arg_string() {
        assert_linter_err!(
            r#"PRINT LEFT$("hello", "world")"#,
            LintError::ArgumentTypeMismatch
        );
    }
}
