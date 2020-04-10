use super::{BlockNode, NameNode, QualifiedNameNode, TypeResolver};
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

    pub fn resolve(&self, resolver: &dyn TypeResolver) -> QualifiedFunctionImplementationNode {
        QualifiedFunctionImplementationNode::new(
            self.name.resolve(resolver),
            self.parameters
                .iter()
                .map(|p| p.resolve(resolver))
                .collect(),
            self.block.clone(),
            self.pos,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedFunctionImplementationNode {
    pub name: QualifiedNameNode,
    pub parameters: Vec<QualifiedNameNode>,
    pub block: BlockNode,
    pub pos: Location,
}

impl QualifiedFunctionImplementationNode {
    pub fn new(
        name: QualifiedNameNode,
        parameters: Vec<QualifiedNameNode>,
        block: BlockNode,
        pos: Location,
    ) -> QualifiedFunctionImplementationNode {
        QualifiedFunctionImplementationNode {
            name,
            parameters,
            block,
            pos,
        }
    }
}
