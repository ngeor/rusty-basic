use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_numeric_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn must_have_arguments() {
        let input = "X = PEEK()";
        assert_linter_err!(input, LintError::FunctionNeedsArguments);
    }

    #[test]
    fn two_arguments() {
        let input = "X = PEEK(1, 4)";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }

    #[test]
    fn string_argument() {
        let input = "X = PEEK(A$)";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }
}
