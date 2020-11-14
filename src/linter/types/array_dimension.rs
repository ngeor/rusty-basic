use crate::linter::ExpressionNode;

pub type ArrayDimensions = Vec<ArrayDimension>;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDimension {
    pub lbound: Option<ExpressionNode>,
    pub ubound: ExpressionNode,
}
