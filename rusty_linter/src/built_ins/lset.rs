use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    // the parser should produce 3 arguments:
    // the variable name, as a string literal
    // the variable itself, a ByRef string variable
    // a string expression to assign to
    if args.len() != 3 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_string_argument(0)?;
    // TODO ensure LSET is operating on variables previously used by FIELD in this scope
    args.require_string_variable(1)?;
    args.require_string_argument(2)
}
