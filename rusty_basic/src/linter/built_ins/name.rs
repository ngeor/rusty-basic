use crate::linter::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() != 2 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        args.require_string_argument(0)?;
        args.require_string_argument(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
    }
}
