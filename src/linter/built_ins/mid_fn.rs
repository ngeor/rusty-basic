use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)
    } else if args.len() == 3 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)?;
        args.require_integer_argument(2)
    } else {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn test_mid_linter() {
        assert_linter_err!(r#"PRINT MID$("oops")"#, QError::ArgumentCountMismatch, 1, 7);
    }
}
