use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    args.require_one_numeric_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_chr() {
        assert_linter_err!("PRINT CHR$(33, 34)", QError::ArgumentCountMismatch, 1, 7);
        assert_linter_err!(r#"PRINT CHR$("33")"#, QError::ArgumentTypeMismatch, 1, 12);
    }
}
