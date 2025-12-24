use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_numeric_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn test_chr() {
        assert_linter_err!("PRINT CHR$(33, 34)", LintError::ArgumentCountMismatch, 1, 7);
        assert_linter_err!(
            r#"PRINT CHR$("33")"#,
            LintError::ArgumentTypeMismatch,
            1,
            12
        );
    }
}
