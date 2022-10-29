use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() != 2 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_integer_argument(0)?;
    args.require_long_argument(1)
}
