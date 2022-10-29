use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() < 2 || args.len() > 4 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn lint_too_many_args() {
        assert_linter_err!("LOCATE 1, 2, 3, 4", QError::ArgumentCountMismatch);
    }
}
