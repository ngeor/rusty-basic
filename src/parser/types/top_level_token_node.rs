use super::{DefTypeNode, FunctionDeclarationNode, FunctionImplementationNode, StatementNode};

pub type ProgramNode = Vec<TopLevelTokenNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelTokenNode {
    FunctionDeclaration(FunctionDeclarationNode),
    Statement(StatementNode),
    FunctionImplementation(FunctionImplementationNode),

    /// A default type definition, e.g. DEFINT A-Z.
    DefType(DefTypeNode),
}
