use crate::linter::arg_validation::ArgValidation;
use rusty_common::QErrorNode;
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_numeric_argument()
}
