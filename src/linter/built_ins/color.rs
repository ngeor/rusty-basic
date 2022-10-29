use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() < 2 || args.len() > 3 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        for i in 0..args.len() {
            args.require_numeric_argument(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;

    #[test]
    fn lint_wrong_foreground_type() {
        let input = "COLOR A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_wrong_background_type() {
        let input = "COLOR , A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_too_many_args() {
        let input = "COLOR 1, 2, 3";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }
}
