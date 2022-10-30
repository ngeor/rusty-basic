use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_string_argument(1)
    } else if args.len() == 3 {
        args.require_integer_argument(0)?;
        args.require_string_argument(1)?;
        args.require_string_argument(2)
    } else {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_instr_linter() {
        assert_linter_err!(
            r#"PRINT INSTR("oops")"#,
            QError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
