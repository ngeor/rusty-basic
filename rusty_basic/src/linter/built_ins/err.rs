use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_zero_arguments()
}
