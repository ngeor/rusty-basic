use super::NameNode;
use crate::common::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclarationNode {
    pub name: NameNode,
    pub parameters: Vec<NameNode>,
    pub pos: Location,
}

impl FunctionDeclarationNode {
    pub fn new(
        name: NameNode,
        parameters: Vec<NameNode>,
        pos: Location,
    ) -> FunctionDeclarationNode {
        FunctionDeclarationNode {
            name,
            parameters,
            pos,
        }
    }
}
