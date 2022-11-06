use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT UCASE$()"#, LintError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT UCASE$("oops", "oops")"#,
            LintError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT UCASE$(10)"#, LintError::ArgumentTypeMismatch);
    }
}
