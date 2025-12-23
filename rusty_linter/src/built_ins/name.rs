use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() != 2 {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    } else {
        args.require_string_argument(0)?;
        args.require_string_argument(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, LintError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, LintError::ArgumentTypeMismatch, 1, 15);
    }
}
