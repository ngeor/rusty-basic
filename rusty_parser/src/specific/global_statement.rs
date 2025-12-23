use rusty_common::*;

use crate::specific::{
    BareName, BareNamePos, DefType, Name, NamePos, Parameter, Parameters, Statement, Statements,
    UserDefinedType,
};

pub type Program = Vec<GlobalStatementPos>;
pub type GlobalStatementPos = Positioned<GlobalStatement>;

/// Represents a parsed token that can appear as a top-level element of the
/// parsing tree.
///
/// Syntax reference
///
/// ```txt
/// <program> ::= <global-statement> | <global-statement><program>
///
/// <global-statement> ::= <comment>
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
pub enum GlobalStatement {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(NamePos, Parameters),

    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(BareNamePos, Parameters),

    /// A sub implementation
    SubImplementation(SubImplementation),

    /// A user defined type definition
    UserDefinedType(UserDefinedType),
}

impl From<Statement> for GlobalStatement {
    fn from(s: Statement) -> Self {
        Self::Statement(s)
    }
}

/// The implementation of a subprogram (FUNCTION or SUB).
#[derive(Clone, Debug, PartialEq)]
pub struct SubprogramImplementation<T> {
    /// The name of the subprogram.
    /// It can be [BareName] for SUBs or [Name] for FUNCTIONs.
    pub name: Positioned<T>,

    /// The parameters of the subprogram.
    pub params: Vec<Positioned<Parameter>>,

    /// The body (statements) of the subprogram.
    pub body: Statements,

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
