use crate::common::*;

use super::{
    BareName, BareNameNode, DefType, Name, NameNode, ParamName, ParamNameNodes, Statement,
    StatementNodes, UserDefinedType,
};

pub type ProgramNode = Vec<TopLevelTokenNode>;
pub type TopLevelTokenNode = Locatable<TopLevelToken>;

/// Represents a parsed token that can appear as a top-level element of the
/// parsing tree.
///
/// Syntax reference
///
/// ```txt
/// <program> ::= <top-level-token> | <top-level-token><program>
///
/// <top-level-token> ::= <comment>
///     | <def-type>
///     | <declaration>
///     | <statement>
///     | <function>
///     | <sub>
///     | <user-defined-type>
///
/// <statement> ::= <comment>
///     | <dim>
///     | <const>
///     | <built-in>
///     | <label>
///     | <sub-call>
///     | <assignment>
///     | <if-block>
///     | <for-loop>
///     | <select-case>
///     | <while-wend>
///     | <go-to>
///     | <on-error-go-to>
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NameNode, ParamNameNodes),

    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNameNode, ParamNameNodes),

    /// A sub implementation
    SubImplementation(SubImplementation),

    /// A user defined type definition
    UserDefinedType(UserDefinedType),
}

impl From<Statement> for TopLevelToken {
    fn from(s: Statement) -> Self {
        TopLevelToken::Statement(s)
    }
}

/// The implementation of a subprogram (FUNCTION or SUB).
#[derive(Clone, Debug, PartialEq)]
pub struct SubprogramImplementation<T> {
    /// The name of the subprogram.
    /// It can be [BareName] for SUBs or [Name] for FUNCTIONs.
    pub name: Locatable<T>,

    /// The parameters of the subprogram.
    pub params: Vec<Locatable<ParamName>>,

    /// The body (statements) of the subprogram.
    pub body: StatementNodes,

    /// Determines if the subprogram is static. Static subprograms retain their
    /// variable values between calls.
    pub is_static: bool,
}

/// The implementation of a SUB.
/// The name type is [BareName] as SUBs don't have a return type.
pub type SubImplementation = SubprogramImplementation<BareName>;

/// The implementation of a FUNCTION.
/// Functions have a built-in return type.
/// The name type is [Name] because the name is not resolved yet.
/// After linting, the name will be resolved.
pub type FunctionImplementation = SubprogramImplementation<Name>;
