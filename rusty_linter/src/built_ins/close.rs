use crate::arg_validation::ArgValidation;
use rusty_common::QErrorNode;
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    for i in 0..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}
