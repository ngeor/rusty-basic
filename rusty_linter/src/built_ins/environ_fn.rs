use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn test_function_call_environ_two_args_linter_err() {
        assert_linter_err!(
            r#"X$ = ENVIRON$("hi", "bye")"#,
            LintError::ArgumentCountMismatch,
            1,
            6
        );
    }

    #[test]
    fn test_function_call_environ_numeric_arg_linter_err() {
        assert_linter_err!("X$ = ENVIRON$(42)", LintError::ArgumentTypeMismatch, 1, 15);
    }
}
