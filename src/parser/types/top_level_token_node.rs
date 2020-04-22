use super::{BareNameNode, BlockNode, DefTypeNode, NameNode, StatementNode};
use crate::common::Location;

pub type ProgramNode = Vec<TopLevelTokenNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelTokenNode {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefTypeNode),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NameNode, Vec<NameNode>, Location),

    /// A function implementation
    FunctionImplementation(NameNode, Vec<NameNode>, BlockNode, Location),

    /// A simple or compound statement
    Statement(StatementNode),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNameNode, Vec<NameNode>, Location),

    /// A sub implementation
    SubImplementation(BareNameNode, Vec<NameNode>, BlockNode, Location),
}
