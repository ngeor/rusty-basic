use crate::common::QErrorNode;
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_numeric_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn must_have_arguments() {
        let input = "X = PEEK()";
        assert_linter_err!(input, QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_arguments() {
        let input = "X = PEEK(1, 4)";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_argument() {
        let input = "X = PEEK(A$)";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
