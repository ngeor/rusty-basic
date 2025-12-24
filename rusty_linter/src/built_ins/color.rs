use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() < 2 || args.len() > 3 {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    } else {
        for i in 0..args.len() {
            args.require_numeric_argument(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn lint_wrong_foreground_type() {
        let input = "COLOR A$";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_wrong_background_type() {
        let input = "COLOR , A$";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_too_many_args() {
        let input = "COLOR 1, 2, 3";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }
}
