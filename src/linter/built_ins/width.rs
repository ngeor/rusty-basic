use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    // 1 or 2 arguments (row, col) but they're duplicated because they're optional
    if args.len() > 4 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}
