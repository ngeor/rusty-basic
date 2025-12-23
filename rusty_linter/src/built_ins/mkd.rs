use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_double_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn no_args() {
        assert_linter_err!("PRINT MKD$()", LintError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT MKD$(A#, B#)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_string() {
        assert_linter_err!("PRINT MKD$(\"10\")", LintError::ArgumentTypeMismatch);
    }
}
