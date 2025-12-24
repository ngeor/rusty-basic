use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn no_args() {
        assert_linter_err!("PRINT CVD()", LintError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT CVD(A$, B$)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_integer() {
        assert_linter_err!("PRINT CVD(10)", LintError::ArgumentTypeMismatch);
    }
}
