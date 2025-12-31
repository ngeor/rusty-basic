use crate::ExpressionPos;

pub type ArrayDimensions = Vec<ArrayDimension>;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDimension {
    pub lbound: Option<ExpressionPos>,
    pub ubound: ExpressionPos,
}
