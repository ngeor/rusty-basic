use crate::common::*;
use crate::linter::QualifiedName;
use crate::variant::Variant;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// Loads a value into register A
    Load(Variant),
    /// Stores a value from register A
    Store(QualifiedName),
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
    //EqualTo,
    LessOrEqualThan,
    LessThan,
    GreaterThan,
    GreaterOrEqualThan,
    NegateA,
    NotA,
    Jump(usize),
    JumpIfFalse(usize),
    Label(CaseInsensitiveString),
    UnresolvedJump(CaseInsensitiveString),
    UnresolvedJumpIfFalse(CaseInsensitiveString),
    CopyVarToA(QualifiedName),
    BuiltInSub(CaseInsensitiveString),
    BuiltInFunction(QualifiedName),
    Halt,

    PushRegisters,
    PopRegisters,

    PushRet(usize),
    PopRet,

    PreparePush,
    PushStack,
    PopStack,

    PushUnnamedRefParam(QualifiedName),

    /// Pushes the contents of register A at the end of the unnamed stack
    PushUnnamedValParam,
    SetNamedRefParam(NamedRefParam),
    SetNamedValParam(QualifiedName),

    Throw(String),

    /// Stores A as the result of a function
    StoreAToResult,
    /// Copies the result of a function to A
    CopyResultToA,

    SetUnresolvedErrorHandler(CaseInsensitiveString),
    SetErrorHandler(usize),
}

pub type InstructionNode = Locatable<Instruction>;

#[derive(Clone, Debug, PartialEq)]
pub struct NamedRefParam {
    pub parameter_name: QualifiedName,
    pub argument_name: QualifiedName,
}
