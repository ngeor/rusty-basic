use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_common::Position;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    args.require_one_variable(pos)
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARSEG()", LintError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARSEG(A, B)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARSEG(3)", LintError::VariableRequired);
    }
}
