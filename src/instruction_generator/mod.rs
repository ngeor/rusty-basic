mod args;
mod built_in_functions;
mod built_in_subs;
mod dim;
mod expression;
mod for_loop;
mod function_call;
mod if_block;
mod instruction_generator;
mod label_resolver;
mod parameter_collector;
pub mod print;
mod select_case;
mod statement;
mod sub_call;
mod while_wend;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;

use crate::built_ins::*;
use crate::common::*;
use crate::instruction_generator::instruction_generator::InstructionGenerator;
use crate::instruction_generator::label_resolver::LabelResolver;
use crate::instruction_generator::parameter_collector::{ParameterCollector, SubProgramParameters};
use crate::parser::{
    BareName, ExpressionType, Name, ParamName, ProgramNode, QualifiedName, TypeQualifier,
};
use crate::variant::Variant;
use print::PrinterType;

/// Generates instructions for the given program.
pub fn generate_instructions(program: ProgramNode) -> InstructionGeneratorResult {
    // pass 1: collect function/sub names -> parameter names, in order to use them in function/sub calls
    // the parameter names and types are needed
    let mut parameter_collector = ParameterCollector::default();
    parameter_collector.visit(&program);
    let sub_program_parameters: SubProgramParameters = parameter_collector.into();
    // pass 2 generate with labels still unresolved
    let mut generator = InstructionGenerator::new(sub_program_parameters);
    generator.generate_unresolved(program);
    let InstructionGenerator {
        instructions,
        statement_addresses,
        ..
    } = generator;
    // pass 3 resolve labels to addresses
    let mut label_resolver = LabelResolver::new(instructions);
    label_resolver.resolve_labels();
    let LabelResolver { instructions } = label_resolver;
    InstructionGeneratorResult {
        instructions,
        statement_addresses,
    }
}

pub struct InstructionGeneratorResult {
    pub instructions: Vec<InstructionNode>,
    pub statement_addresses: Vec<usize>,
}

#[derive(Debug)]
pub enum Path {
    Root(RootPath),
    ArrayElement(Box<Path>, Vec<Variant>),
    Property(Box<Path>, BareName),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RootPath {
    /// The name of the root variable
    pub name: Name,

    /// If true, the variable belongs to the global shared context,
    /// i.e. it was declared with DIM SHARED
    pub shared: bool,
}

impl Path {
    pub fn append_array_element(self, index: Variant) -> Self {
        match self {
            Self::Root(root_path) => {
                Self::ArrayElement(Box::new(Self::Root(root_path)), vec![index])
            }
            Self::ArrayElement(parent, mut indices) => {
                indices.push(index);
                Self::ArrayElement(parent, indices)
            }
            _ => panic!("unexpected NamePtr"),
        }
    }
}

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

    Label(CaseInsensitiveString),

    Jump(AddressOrLabel),

    JumpIfFalse(AddressOrLabel),

    GoSub(AddressOrLabel),

    Return(Option<AddressOrLabel>),

    Resume,
    ResumeNext,
    ResumeLabel(AddressOrLabel),

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

    OnErrorGoTo(AddressOrLabel),
    OnErrorResumeNext,
    OnErrorGoToZero,

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

    PrintSetPrinterType(PrinterType),
    PrintSetFileHandle(FileHandle),
    PrintSetFormatStringFromA,
    PrintComma,
    PrintSemicolon,
    PrintValueFromA,
    PrintEnd,
}

pub type InstructionNode = Locatable<Instruction>;

#[derive(Clone, Debug, PartialEq)]
pub enum AddressOrLabel {
    Resolved(usize),
    Unresolved(CaseInsensitiveString),
}

impl AddressOrLabel {
    pub fn address(&self) -> usize {
        if let Self::Resolved(address) = self {
            *address
        } else {
            panic!("Unresolved label")
        }
    }
}
