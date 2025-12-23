use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.is_empty() {
        Ok(())
    } else if args.len() == 1 {
        args.require_numeric_argument(0)
    } else {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn lint_arg_wrong_type() {
        let input = "CLS A$";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_two_args() {
        let input = "CLS 42, 1";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }
}
