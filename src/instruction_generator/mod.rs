mod calls;
mod dim;
mod expression;
mod for_loop;
mod if_block;
mod label_resolver;
pub mod print;
mod select_case;
mod statement;
mod subprogram_info;
mod while_wend;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;

use crate::built_ins::*;
use crate::common::*;
use crate::instruction_generator::label_resolver::LabelResolver;
use crate::instruction_generator::subprogram_info::{
    SubprogramInfoCollector, SubprogramInfoRepository,
};
use crate::linter::SubprogramName;
use crate::parser::*;
use crate::variant::Variant;

/// Generates instructions for the given program.
pub fn generate_instructions(program: ProgramNode) -> InstructionGeneratorResult {
    // pass 1: collect function/sub names -> parameter names, in order to use them in function/sub calls
    // the parameter names and types are needed
    let mut subprogram_info_collector = SubprogramInfoCollector::default();
    subprogram_info_collector.visit(&program);
    let subprogram_parameters: SubprogramInfoRepository = subprogram_info_collector.into();
    // pass 2 generate with labels still unresolved
    let mut generator = InstructionGenerator::new(subprogram_parameters);
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
    PushStaticStack(SubprogramName),
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

    /// Checks if a variable is defined (used to prevent re-allocation of variables in STATIC functions/subs).
    /// If the variable is already present, it will set the A register to true, otherwise to false.
    IsVariableDefined(DimName),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrinterType {
    Print,
    LPrint,
    File,
}

struct InstructionGenerator {
    instructions: Vec<InstructionNode>,
    statement_addresses: Vec<usize>,
    subprogram_info_repository: SubprogramInfoRepository,
    current_subprogram: Option<SubprogramName>,
}

impl InstructionGenerator {
    fn new(subprogram_info_repository: SubprogramInfoRepository) -> Self {
        Self {
            instructions: vec![],
            statement_addresses: vec![],
            subprogram_info_repository,
            current_subprogram: None,
        }
    }

    fn generate_unresolved(&mut self, program: ProgramNode) {
        let (top_level_statements, functions, subs) = Self::split_program(program);
        self.visit_top_level_statements(top_level_statements);
        self.visit_functions(functions);
        self.visit_subs(subs);
    }

    fn push(&mut self, i: Instruction, pos: Location) {
        self.instructions.push(i.at(pos));
    }

    fn split_program(
        program: ProgramNode,
    ) -> (
        StatementNodes,
        Vec<Locatable<FunctionImplementation>>,
        Vec<Locatable<SubImplementation>>,
    ) {
        let mut top_level_statements: StatementNodes = vec![];
        let mut functions: Vec<Locatable<FunctionImplementation>> = vec![];
        let mut subs: Vec<Locatable<SubImplementation>> = vec![];
        for Locatable { element, pos } in program {
            match element {
                TopLevelToken::Statement(s) => {
                    top_level_statements.push(s.at(pos));
                }
                TopLevelToken::FunctionImplementation(f) => {
                    functions.push(f.at(pos));
                }
                TopLevelToken::SubImplementation(s) => {
                    subs.push(s.at(pos));
                }
                _ => {}
            }
        }
        (top_level_statements, functions, subs)
    }

    fn visit_top_level_statements(&mut self, statements: StatementNodes) {
        self.generate_block_instructions(statements);

        // add HALT instruction at end of program to separate from the functions and subs
        self.mark_statement_address();
        self.push(
            Instruction::Halt,
            Location::new(std::u32::MAX, std::u32::MAX),
        );
    }

    fn visit_functions(&mut self, functions: Vec<Locatable<FunctionImplementation>>) {
        for f in functions {
            self.visit_function(f);
        }
    }

    fn visit_function(&mut self, function_node: Locatable<FunctionImplementation>) {
        let Locatable {
            element: function_implementation,
            pos,
        } = function_node;
        let FunctionImplementation { name, body, .. } = function_implementation;
        let function_name = name.element.demand_qualified();
        self.mark_current_subprogram(SubprogramName::Function(function_name.clone()), pos);
        // set default value
        self.push_load(function_name.qualifier, pos);
        self.subprogram_body(body, pos);
    }

    fn visit_subs(&mut self, subs: Vec<Locatable<SubImplementation>>) {
        for s in subs {
            self.visit_sub(s);
        }
    }

    fn visit_sub(&mut self, sub_node: Locatable<SubImplementation>) {
        let Locatable {
            element: sub_implementation,
            pos,
        } = sub_node;
        let SubImplementation {
            name: Locatable { element: name, .. },
            body,
            ..
        } = sub_implementation;
        self.mark_current_subprogram(SubprogramName::Sub(name.clone()), pos);
        self.subprogram_body(body, pos);
    }

    fn mark_current_subprogram(&mut self, subprogram_name: SubprogramName, pos: Location) {
        self.push(
            Instruction::Label(Self::format_subprogram_label(&subprogram_name)),
            pos,
        );
        self.current_subprogram = Some(subprogram_name);
    }

    fn subprogram_body(&mut self, block: StatementNodes, pos: Location) {
        self.generate_block_instructions(block);
        // to be able to RESUME NEXT if an error occurs on the last statement
        self.mark_statement_address();
        self.push(Instruction::PopRet, pos);
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A.
    fn push_load<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push(Instruction::LoadIntoA(value.into()), pos);
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A, followed by a PushUnnamed instruction.
    fn push_load_unnamed_arg<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push_load(value, pos);
        self.push(Instruction::PushAToUnnamedArg, pos);
    }

    fn jump_if_false<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    fn jump<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    fn label<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                "_{}_{:?}",
                prefix.as_ref(),
                pos
            ))),
            pos,
        );
    }

    fn generate_assignment_instructions(
        &mut self,
        l: Expression,
        r: ExpressionNode,
        pos: Location,
    ) {
        let left_type = l.expression_type();
        self.generate_expression_instructions_casting(r, left_type);
        self.generate_store_instructions(l, pos);
    }

    fn generate_store_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyAToVarPath, pos);
    }

    fn generate_load_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyVarPathToA, pos);
    }

    fn mark_statement_address(&mut self) {
        self.statement_addresses.push(self.instructions.len());
    }

    fn format_subprogram_label(subprogram_name: &SubprogramName) -> BareName {
        let s: String = match subprogram_name {
            SubprogramName::Function(function_name) => {
                let mut s: String = String::new();
                s.push_str(":fun:");
                s.push_str(&function_name.to_string());
                s
            }
            SubprogramName::Sub(sub_name) => {
                let mut s: String = String::new();
                s.push_str(":sub:");
                s.push_str(sub_name.as_ref());
                s
            }
        };
        BareName::new(s)
    }
}
