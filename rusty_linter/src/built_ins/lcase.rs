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
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LCASE$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT LCASE$("oops", "oops")"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LCASE$(10)"#, QError::ArgumentTypeMismatch);
    }
}
