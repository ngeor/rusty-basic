use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::{QualifiedName, ResolvedDeclaredName, TypeQualifier};
use crate::variant::Variant;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Dim(ResolvedDeclaredName),
    /// Loads a value into register A
    Load(Variant),
    /// Stores a value from register A
    Store(ResolvedDeclaredName),
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
    CopyVarToA(ResolvedDeclaredName),
    BuiltInSub(BuiltInSub),
    BuiltInFunction(BuiltInFunction),
    Halt,

    PushRegisters,
    PopRegisters,

    PushRet(usize),
    PopRet,

    PreparePush,
    PushStack,
    PopStack,

    PushUnnamedRefParam(ResolvedDeclaredName),

    /// Pushes the contents of register A at the end of the unnamed stack
    PushUnnamedValParam,
    SetNamedRefParam(NamedRefParam),
    SetNamedValParam(ResolvedDeclaredName),

    Throw(QError),

    /// Stores A as the result of a function
    StoreAToResult,
    /// Copies the result of a function to A
    CopyResultToA,

    SetUnresolvedErrorHandler(CaseInsensitiveString),
    SetErrorHandler(usize),

    /// Cast the contents of A into the given type
    Cast(TypeQualifier),
}

pub type InstructionNode = Locatable<Instruction>;

#[derive(Clone, Debug, PartialEq)]
pub struct NamedRefParam {
    /// The name of the parameter as expected in the function or sub
    pub parameter_name: ResolvedDeclaredName,

    /// The name of the argument as passed at the call location
    pub argument_name: ResolvedDeclaredName,
}
