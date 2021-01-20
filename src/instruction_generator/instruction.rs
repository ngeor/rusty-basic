use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::parser::{BareName, ExpressionType, ParamName, QualifiedName, TypeQualifier};
use crate::variant::{RootPath, Variant};

#[derive(Debug, PartialEq)]
pub enum Instruction {
    // Storing into variables is done in two steps:
    // the first step is to evaluate the variable path.
    // For a simple variable, that's just the variable name,
    // which can be unqualified for user defined types and
    // qualified for built-in types.
    // The second step is to write the register A into the variant that the
    // variable path references.
    VarPathName(RootPath),

    VarPathIndex,

    VarPathProperty(BareName),

    /// Copies the value of register A into the variable path
    CopyAToVarPath,

    /// Copies the value of the variable path into register A
    CopyVarPathToA,

    /// Loads a value into register A
    LoadIntoA(Variant),

    CopyAToB,
    CopyAToC,
    CopyAToD,
    CopyCToB,
    CopyDToA,
    CopyDToB,
    /// Adds registers A and B and stores the results into register A
    Plus,
    Minus,
    Multiply,
    Divide,
    Less,
    LessOrEqual,
    Equal,
    GreaterOrEqual,
    Greater,
    NotEqual,
    NegateA,
    NotA,
    And,
    Or,
    Jump(usize),
    JumpIfFalse(usize),
    Label(CaseInsensitiveString),
    UnresolvedJump(CaseInsensitiveString),
    UnresolvedJumpIfFalse(CaseInsensitiveString),

    BuiltInSub(BuiltInSub),
    BuiltInFunction(BuiltInFunction),
    Halt,

    PushRegisters,
    PopRegisters,

    PushAToValueStack,
    PopValueStackIntoA,

    PushRet(usize),
    PopRet,

    /// Starts collecting arguments.
    ///
    /// Arguments are evaluated within the current naming context and pushed with
    /// PushNamed and PushUnnamed.
    BeginCollectArguments,

    /// Pushes the value of register A as a named parameter to a child context.
    PushNamed(ParamName),

    /// Pushes the value of register A as an unnamed parameter to a child context.
    /// Unnamed parameters are used by built-in functions/subs.
    PushAToUnnamedArg,

    PushStack,
    PopStack,

    EnqueueToReturnStack(usize),
    DequeueFromReturnStack,

    StashFunctionReturnValue(QualifiedName),
    UnStashFunctionReturnValue,

    Throw(QError),

    SetUnresolvedErrorHandler(CaseInsensitiveString),
    SetErrorHandler(usize),

    /// Cast the contents of A into the given type
    Cast(TypeQualifier),

    FixLength(u16),

    // allocating variables
    AllocateBuiltIn(TypeQualifier),

    AllocateFixedLengthString(u16),

    /// Allocates an array of the given type. The dimensions need to have been
    /// first pushed with `PushUnnamed`.
    AllocateArrayIntoA(ExpressionType),

    AllocateUserDefined(BareName),
}

pub type InstructionNode = Locatable<Instruction>;
