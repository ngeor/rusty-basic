use super::{
    BareNameNode, DeclaredNameNodes, DefType, NameNode, Statement, StatementNodes, UserDefinedType,
};
use crate::common::*;

pub type ProgramNode = Vec<TopLevelTokenNode>;
pub type TopLevelTokenNode = Locatable<TopLevelToken>;

pub type DotlessName = CaseInsensitiveString;
pub enum Param {
    Bare(CaseInsensitiveString),
    // Compact(QualifiedName),
    // parameters do not allow for STRING * 5
    // Extended(QualifiedName)
    // UserDefined(DotlessName, DotlessName)
}

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NameNode, DeclaredNameNodes),

    /// A function implementation
    FunctionImplementation(NameNode, DeclaredNameNodes, StatementNodes),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNameNode, DeclaredNameNodes),

    /// A sub implementation
    SubImplementation(BareNameNode, DeclaredNameNodes, StatementNodes),

    /// A user defined type definition
    UserDefinedType(UserDefinedType),
}

impl From<Statement> for TopLevelToken {
    fn from(s: Statement) -> Self {
        TopLevelToken::Statement(s)
    }
}
