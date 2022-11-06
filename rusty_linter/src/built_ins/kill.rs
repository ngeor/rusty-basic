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
    fn test_kill_linter() {
        assert_linter_err!("KILL", LintError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL "a", "b""#, LintError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL 42"#, LintError::ArgumentTypeMismatch, 1, 6);
    }
}
