use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.is_empty() {
        Ok(())
    } else {
        args.require_one_numeric_argument()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn address_cannot_be_string() {
        let input = "DEF SEG = A$";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }
}
