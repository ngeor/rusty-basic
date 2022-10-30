use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.is_empty() {
        Ok(())
    } else if args.len() == 1 {
        args.require_numeric_argument(0)
    } else {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn lint_arg_wrong_type() {
        let input = "CLS A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_two_args() {
        let input = "CLS 42, 1";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }
}
