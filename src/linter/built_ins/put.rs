
use crate::common::QErrorNode;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    super::get::lint(args)
}
