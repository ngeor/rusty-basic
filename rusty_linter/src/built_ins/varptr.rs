use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_variable()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARPTR()", LintError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARPTR(A, B)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARPTR(3)", LintError::VariableRequired);
    }
}
