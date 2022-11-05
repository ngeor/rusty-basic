use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    args.require_one_variable()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARSEG()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARSEG(A, B)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARSEG(3)", QError::VariableRequired);
    }
}
