use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    for i in 0..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}
