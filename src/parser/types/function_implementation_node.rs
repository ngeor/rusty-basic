use super::{BlockNode, NameNode};
use crate::common::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionImplementationNode {
    pub name: NameNode,
    pub parameters: Vec<NameNode>,
    pub block: BlockNode,
    pub pos: Location,
}

impl FunctionImplementationNode {
    pub fn new(
        name: NameNode,
        parameters: Vec<NameNode>,
        block: BlockNode,
        pos: Location,
    ) -> FunctionImplementationNode {
        FunctionImplementationNode {
            name,
            parameters,
            block,
            pos,
        }
    }
}
