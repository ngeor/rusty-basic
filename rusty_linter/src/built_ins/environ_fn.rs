use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_function_call_environ_two_args_linter_err() {
        assert_linter_err!(
            r#"X$ = ENVIRON$("hi", "bye")"#,
            QError::ArgumentCountMismatch,
            1,
            6
        );
    }

    #[test]
    fn test_function_call_environ_numeric_arg_linter_err() {
        assert_linter_err!("X$ = ENVIRON$(42)", QError::ArgumentTypeMismatch, 1, 15);
    }
}
