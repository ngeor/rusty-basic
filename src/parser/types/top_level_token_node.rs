use super::{BareNameNode, DefType, NameNode, Statement, StatementNodes};
use crate::common::*;

pub type ProgramNode = Vec<TopLevelTokenNode>;
pub type TopLevelTokenNode = Locatable<TopLevelToken>;
pub type ParamNodes = Vec<NameNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NameNode, ParamNodes),

    /// A function implementation
    FunctionImplementation(NameNode, ParamNodes, StatementNodes),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNameNode, ParamNodes),

    /// A sub implementation
    SubImplementation(BareNameNode, ParamNodes, StatementNodes),
}

impl From<Statement> for TopLevelToken {
    fn from(s: Statement) -> Self {
        TopLevelToken::Statement(s)
    }
}
