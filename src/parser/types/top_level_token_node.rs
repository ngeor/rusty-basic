use super::{FunctionDeclarationNode, FunctionImplementationNode, StatementNode, TopLevelToken};
use crate::common::StripLocation;

pub type ProgramNode = Vec<TopLevelTokenNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelTokenNode {
    FunctionDeclaration(FunctionDeclarationNode),
    Statement(StatementNode),
    FunctionImplementation(FunctionImplementationNode),
}

impl StripLocation<TopLevelToken> for TopLevelTokenNode {
    fn strip_location(self) -> TopLevelToken {
        match self {
            TopLevelTokenNode::FunctionDeclaration(n) => TopLevelToken::FunctionDeclaration(
                n.name.strip_location(),
                n.parameters.strip_location(),
            ),
            TopLevelTokenNode::Statement(s) => TopLevelToken::Statement(s.strip_location()),
            TopLevelTokenNode::FunctionImplementation(n) => TopLevelToken::FunctionImplementation(
                n.name.strip_location(),
                n.parameters.strip_location(),
                n.block.strip_location(),
            ),
        }
    }
}
