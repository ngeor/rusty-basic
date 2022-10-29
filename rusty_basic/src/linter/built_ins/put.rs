use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    super::get::lint(args)
}
