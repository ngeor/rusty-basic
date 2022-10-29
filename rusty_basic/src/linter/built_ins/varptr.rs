use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_variable()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARPTR()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARPTR(A, B)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARPTR(3)", QError::VariableRequired);
    }
}
