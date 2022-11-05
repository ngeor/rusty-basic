use crate::instruction_generator::label_resolver::LabelResolver;
use crate::instruction_generator::subprogram_info::{
    SubprogramInfoCollector, SubprogramInfoRepository,
};
use rusty_common::{AtPos, CaseInsensitiveString, Position, Positioned, QError};
use rusty_linter::SubprogramName;
use rusty_parser::*;
use rusty_variant::Variant;

/// Generates instructions for the given program.
pub fn generate_instructions(program: Program) -> InstructionGeneratorResult {
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
    pub instructions: Vec<InstructionPos>,
    pub statement_addresses: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum Path {
    Root(RootPath),
    ArrayElement(Box<Path>, Vec<Variant>),
    Property(Box<Path>, BareName),
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Copies the value of register A into the variable path.
    ///
    /// The variable path is automatically dropped by the var path deque,
    /// i.e. the `PopVarPath` instruction is implicitly executed.
    CopyAToVarPath,

    /// Copies the value of the variable path into register A.
    ///
    /// The variable path is not dropped from the var path deque, in case it is
    /// needed by the `PushUnnamedByRef` instruction.
    CopyVarPathToA,

    /// Pops a value from the var path deque.
    PopVarPath,

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
    Modulo,
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
    PushNamed(Parameter),

    /// Pushes the value of register A as an unnamed parameter to a child context.
    /// Unnamed parameters are used by built-in functions/subs.
    PushUnnamedByVal,

    /// Pushes the value of register A as an unnamed parameter to a child context.
    /// Additionally, it pops the var path (implicitly uses `PopVarPath`) and pushes
    /// the path that can be used by the built-in function/sub.
    PushUnnamedByRef,

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

    // TODO #[deprecated]
    PrintSetPrinterType(PrinterType),
    // TODO #[deprecated]
    PrintSetFileHandle(FileHandle),
    // TODO #[deprecated]
    PrintSetFormatStringFromA,
    // TODO #[deprecated]
    PrintComma,
    // TODO #[deprecated]
    PrintSemicolon,
    // TODO #[deprecated]
    PrintValueFromA,
    // TODO #[deprecated]
    PrintEnd,

    /// Checks if a variable is defined (used to prevent re-allocation of variables in STATIC functions/subs).
    /// If the variable is already present, it will set the A register to true, otherwise to false.
    IsVariableDefined(DimVar),
}

pub type InstructionPos = Positioned<Instruction>;

#[derive(Clone, Debug, PartialEq, Eq)]
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

// TODO visibility needs to be reduced. It was not pub when it lived in the mess of `mod.rs`
pub struct InstructionGenerator {
    pub instructions: Vec<InstructionPos>,
    pub statement_addresses: Vec<usize>,
    pub subprogram_info_repository: SubprogramInfoRepository,
    pub current_subprogram: Option<SubprogramName>,
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

    fn generate_unresolved(&mut self, program: Program) {
        let (global_statements, functions, subs) = Self::split_program(program);
        self.visit_global_statements(global_statements);
        self.visit_functions(functions);
        self.visit_subs(subs);
    }

    pub fn push(&mut self, i: Instruction, pos: Position) {
        self.instructions.push(i.at_pos(pos));
    }

    fn split_program(
        program: Program,
    ) -> (
        Statements,
        Vec<Positioned<FunctionImplementation>>,
        Vec<Positioned<SubImplementation>>,
    ) {
        let mut global_statements: Statements = vec![];
        let mut functions: Vec<Positioned<FunctionImplementation>> = vec![];
        let mut subs: Vec<Positioned<SubImplementation>> = vec![];
        for Positioned { element, pos } in program {
            match element {
                GlobalStatement::Statement(s) => {
                    global_statements.push(s.at_pos(pos));
                }
                GlobalStatement::FunctionImplementation(f) => {
                    functions.push(f.at_pos(pos));
                }
                GlobalStatement::SubImplementation(s) => {
                    subs.push(s.at_pos(pos));
                }
                _ => {}
            }
        }
        (
            Self::move_data_statements_first(global_statements),
            functions,
            subs,
        )
    }

    fn move_data_statements_first(statements: Statements) -> Statements {
        let mut data_statements: Statements = vec![];
        let mut other_statements: Statements = vec![];
        for statement in statements {
            if let Statement::BuiltInSubCall(BuiltInSub::Data, _) = &statement.element {
                data_statements.push(statement);
            } else {
                other_statements.push(statement);
            }
        }
        data_statements.append(&mut other_statements);
        data_statements
    }

    fn visit_global_statements(&mut self, statements: Statements) {
        self.visit(statements);

        // add HALT instruction at end of program to separate from the functions and subs
        self.mark_statement_address();
        self.push(Instruction::Halt, Position::new(u32::MAX, u32::MAX));
    }

    fn visit_functions(&mut self, functions: Vec<Positioned<FunctionImplementation>>) {
        for f in functions {
            self.visit_function(f);
        }
    }

    fn visit_function(&mut self, function_implementation_pos: Positioned<FunctionImplementation>) {
        let Positioned {
            element: function_implementation,
            pos,
        } = function_implementation_pos;
        let FunctionImplementation {
            name:
                Positioned {
                    element: function_name,
                    ..
                },
            body,
            ..
        } = function_implementation;
        match function_name {
            Name::Bare(_) => panic!("Expected qualified function name"),
            Name::Qualified(bare_name, qualifier) => {
                let qualifier_copy = qualifier;
                let q_function_name = QualifiedName::new(bare_name, qualifier);
                self.mark_current_subprogram(SubprogramName::Function(q_function_name), pos);
                // set default value
                self.push(Instruction::AllocateBuiltIn(qualifier_copy), pos);
                self.subprogram_body(body, pos);
            }
        }
    }

    fn visit_subs(&mut self, subs: Vec<Positioned<SubImplementation>>) {
        for s in subs {
            self.visit_sub(s);
        }
    }

    fn visit_sub(&mut self, sub_implementation_pos: Positioned<SubImplementation>) {
        let Positioned {
            element: sub_implementation,
            pos,
        } = sub_implementation_pos;
        let SubImplementation {
            name: Positioned { element: name, .. },
            body,
            ..
        } = sub_implementation;
        self.mark_current_subprogram(SubprogramName::Sub(name), pos);
        self.subprogram_body(body, pos);
    }

    fn mark_current_subprogram(&mut self, subprogram_name: SubprogramName, pos: Position) {
        self.push(
            Instruction::Label(Self::format_subprogram_label(&subprogram_name)),
            pos,
        );
        self.current_subprogram = Some(subprogram_name);
    }

    fn subprogram_body(&mut self, block: Statements, pos: Position) {
        self.visit(block);
        // to be able to RESUME NEXT if an error occurs on the last statement
        self.mark_statement_address();
        self.push(Instruction::PopRet, pos);
    }

    /// Adds a Load instruction, storing the given [Variant] in register A.
    pub fn push_load(&mut self, value: Variant, pos: Position) {
        self.push(Instruction::LoadIntoA(value), pos);
    }

    /// Adds a Load instruction, storing the given [Variant] in register A,
    /// followed by a PushUnnamed instruction.
    pub fn push_load_unnamed_arg(&mut self, value: Variant, pos: Position) {
        self.push_load(value, pos);
        self.push(Instruction::PushUnnamedByVal, pos);
    }

    pub fn jump_if_false(&mut self, prefix: &str, pos: Position) {
        self.push(
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix, pos),
            ))),
            pos,
        );
    }

    pub fn jump(&mut self, prefix: &str, pos: Position) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix, pos),
            ))),
            pos,
        );
    }

    pub fn label(&mut self, prefix: &str, pos: Position) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!("_{}_{:?}", prefix, pos))),
            pos,
        );
    }

    pub fn generate_assignment_instructions(
        &mut self,
        l: Expression,
        r: ExpressionPos,
        pos: Position,
    ) {
        let left_type = l.expression_type();
        self.generate_expression_instructions_casting(r, left_type);
        self.generate_store_instructions(l, pos);
    }

    pub fn generate_store_instructions(&mut self, l: Expression, pos: Position) {
        self.generate_path_instructions(l.at_pos(pos));
        self.push(Instruction::CopyAToVarPath, pos);
    }

    pub fn mark_statement_address(&mut self) {
        self.statement_addresses.push(self.instructions.len());
    }

    pub fn format_subprogram_label(subprogram_name: &SubprogramName) -> BareName {
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

pub trait Visitor<T> {
    fn visit(&mut self, item: T);
}
