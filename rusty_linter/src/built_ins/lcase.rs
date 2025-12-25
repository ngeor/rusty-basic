use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_common::Position;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    args.require_one_string_argument(pos)
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LCASE$()"#, LintError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT LCASE$("oops", "oops")"#,
            LintError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LCASE$(10)"#, LintError::ArgumentTypeMismatch);
    }
}
