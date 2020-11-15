use crate::common::Locatable;
use crate::linter::types::{ParamName, Statement, StatementNodes};
use crate::linter::UserDefinedType;
use crate::parser::{BareNameNode, DefType, NameNode};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionImplementation {
    pub name: NameNode,
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
    DefType(DefType),

    FunctionDeclaration(NameNode, Vec<Locatable<ParamName>>),

    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    SubDeclaration(BareNameNode, Vec<Locatable<ParamName>>),

    /// A sub implementation
    SubImplementation(SubImplementation),

    UserDefinedType(UserDefinedType),
}

pub type TopLevelTokenNode = Locatable<TopLevelToken>;
pub type ProgramNode = Vec<TopLevelTokenNode>;
