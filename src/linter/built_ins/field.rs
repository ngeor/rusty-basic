use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    // needs to be 1 + N*3 args, N >= 1
    // first is the file number
    // then the fields: width, variable name, variable
    if args.len() < 4 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    if (args.len() - 1) % 3 != 0 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_integer_argument(0)?;
    let mut i: usize = 1;
    while i < args.len() {
        args.require_integer_argument(i)?;
        args.require_string_argument(i + 1)?;
        args.require_string_variable(i + 2)?;
        i += 3;
    }
    Ok(())
}
