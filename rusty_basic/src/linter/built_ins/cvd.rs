use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn no_args() {
        assert_linter_err!("PRINT CVD()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT CVD(A$, B$)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_integer() {
        assert_linter_err!("PRINT CVD(10)", QError::ArgumentTypeMismatch);
    }
}
