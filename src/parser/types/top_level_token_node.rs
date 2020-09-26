use super::{
    BareNameNode, DefType, NameNode, ParamNameNodes, Statement, StatementNodes, UserDefinedType,
};
use crate::common::*;

pub type ProgramNode = Vec<TopLevelTokenNode>;
pub type TopLevelTokenNode = Locatable<TopLevelToken>;

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NameNode, ParamNameNodes),

    /// A function implementation
    FunctionImplementation(NameNode, ParamNameNodes, StatementNodes),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNameNode, ParamNameNodes),

    /// A sub implementation
    SubImplementation(BareNameNode, ParamNameNodes, StatementNodes),

    /// A user defined type definition
    UserDefinedType(UserDefinedType),
}

impl From<Statement> for TopLevelToken {
    fn from(s: Statement) -> Self {
        TopLevelToken::Statement(s)
    }
}
