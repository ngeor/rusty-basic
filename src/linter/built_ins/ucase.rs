use crate::common::QErrorNode;
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT UCASE$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT UCASE$("oops", "oops")"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT UCASE$(10)"#, QError::ArgumentTypeMismatch);
    }
}
