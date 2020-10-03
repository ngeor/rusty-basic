use crate::common::Locatable;
use crate::linter::types::{ParamName, Statement, StatementNodes};
use crate::parser::{BareNameNode, QualifiedNameNode};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionImplementation {
    pub name: QualifiedNameNode,
    pub params: Vec<Locatable<ParamName>>,
    pub body: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubImplementation {
    pub name: BareNameNode,
    pub params: Vec<Locatable<ParamName>>,
    pub body: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub implementation
    SubImplementation(SubImplementation),
}

pub type TopLevelTokenNode = Locatable<TopLevelToken>;
pub type ProgramNode = Vec<TopLevelTokenNode>;
