use rusty_common::QErrorNode;
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    super::get::lint(args)
}
