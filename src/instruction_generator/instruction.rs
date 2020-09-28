use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::{DimName, ParamName};
use crate::parser::{QualifiedName, TypeQualifier};
use crate::variant::Variant;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Dim(DimName),
    /// Loads a value into register A
    Load(Variant),
    /// Stores a value from register A
    Store(DimName),
    /// Stores a value from register A into a constant
    StoreConst(QualifiedName),
    CopyAToB,
    CopyAToC,
    CopyAToD,
    CopyCToB,
    CopyDToA,
    CopyDToB,
    SwapAWithB,
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
    CopyVarToA(DimName),
    BuiltInSub(BuiltInSub),
    BuiltInFunction(BuiltInFunction),
    Halt,

    PushRegisters,
    PopRegisters,

    PushRet(usize),
    PopRet,

    /// Starts collecting named arguments.
    ///
    /// Arguments are evaluated within the current naming context and pushed with
    /// PushNamed.
    BeginCollectNamedArguments,

    /// Starts collecting unnamed arguments (for a built-in sub or function).
    ///
    /// Arguments are evaluated within the current naming context and pushed with
    /// PushUnnamed.
    BeginCollectUnnamedArguments,

    /// Pushes the value of register A as a named parameter to a child context.
    PushNamed(ParamName),

    /// Pushes the value of register A as an unnamed parameter to a child context.
    /// Unnamed parameters are used by built-in functions/subs.
    PushUnnamed,

    PushStack,
    PopStack(Option<QualifiedName>),

    /// Copies the value of a ByRef argument back to the parent context.
    ///
    /// The first item in the tuple is the index of the parameter in the function/sub call.
    /// The second item in the tuple is the name of the variable in the parent context.
    ///
    /// Called just before the child context is popped.
    CopyToParent(usize, DimName),

    Throw(QError),

    SetUnresolvedErrorHandler(CaseInsensitiveString),
    SetErrorHandler(usize),

    /// Cast the contents of A into the given type
    Cast(TypeQualifier),
    FixLength(u16),
}

pub type InstructionNode = Locatable<Instruction>;
