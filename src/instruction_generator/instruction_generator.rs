use crate::common::*;
use crate::instruction_generator::parameter_collector::SubProgramParameters;
use crate::instruction_generator::{AddressOrLabel, Instruction, InstructionNode};
use crate::parser::{
    BareName, Expression, ExpressionNode, FunctionImplementation, HasExpressionType, Name,
    ProgramNode, QualifiedName, SubImplementation, TopLevelToken,
};
use crate::variant::Variant;

pub struct InstructionGenerator {
    pub instructions: Vec<InstructionNode>,
    pub statement_addresses: Vec<usize>,
    pub sub_program_parameters: SubProgramParameters,
}

impl InstructionGenerator {
    pub fn new(sub_program_parameters: SubProgramParameters) -> Self {
        Self {
            instructions: vec![],
            statement_addresses: vec![],
            sub_program_parameters,
        }
    }

    pub fn generate_unresolved(&mut self, program: ProgramNode) {
        let mut functions: Vec<(FunctionImplementation, Location)> = vec![];
        let mut subs: Vec<(SubImplementation, Location)> = vec![];

        for t in program {
            let Locatable {
                element: top_level_token,
                pos,
            } = t;
            match top_level_token {
                TopLevelToken::Statement(s) => {
                    self.generate_statement_node_instructions(s.at(pos));
                }
                TopLevelToken::FunctionImplementation(f) => functions.push((f, pos)),
                TopLevelToken::SubImplementation(s) => subs.push((s, pos)),
                _ => {}
            }
        }

        // add HALT instruction at end of program to separate from the functions and subs
        self.mark_statement_address();
        self.push(
            Instruction::Halt,
            Location::new(std::u32::MAX, std::u32::MAX),
        );

        // functions
        for (f, pos) in functions {
            let Locatable {
                element: function_name,
                ..
            } = f.name;
            if let Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) = function_name
            {
                let block = f.body;
                self.function_label(&bare_name, pos);
                // set default value
                self.push_load(qualifier, pos);
                self.generate_block_instructions(block);
                self.mark_statement_address();
                self.push(Instruction::PopRet, pos);
            } else {
                panic!("Unexpected bare function name {:?}", function_name);
            }
        }

        // subs
        for (s, pos) in subs {
            let name = s.name;
            let bare_name: &BareName = name.as_ref();
            let block = s.body;
            self.sub_label(bare_name, pos);
            self.generate_block_instructions(block);
            self.mark_statement_address();
            self.push(Instruction::PopRet, pos);
        }
    }

    pub fn push(&mut self, i: Instruction, pos: Location) {
        self.instructions.push(i.at(pos));
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A.
    pub fn push_load<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push(Instruction::LoadIntoA(value.into()), pos);
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A, followed by a PushUnnamed instruction.
    pub fn push_load_unnamed_arg<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push_load(value, pos);
        self.push(Instruction::PushAToUnnamedArg, pos);
    }

    pub fn jump_if_false<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    pub fn jump<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    pub fn label<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                "_{}_{:?}",
                prefix.as_ref(),
                pos
            ))),
            pos,
        );
    }

    pub fn function_label<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                ":fun:{}",
                name.as_ref(),
            ))),
            pos,
        );
    }

    pub fn jump_to_function<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!(":fun:{}", name.as_ref(),),
            ))),
            pos,
        );
    }

    pub fn sub_label<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                ":sub:{}",
                name.as_ref(),
            ))),
            pos,
        );
    }

    pub fn jump_to_sub<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!(":sub:{}", name.as_ref(),),
            ))),
            pos,
        );
    }

    pub fn generate_assignment_instructions(
        &mut self,
        l: Expression,
        r: ExpressionNode,
        pos: Location,
    ) {
        let left_type = l.expression_type();
        self.generate_expression_instructions_casting(r, left_type);
        self.generate_store_instructions(l, pos);
    }

    pub fn generate_store_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyAToVarPath, pos);
    }

    pub fn generate_load_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyVarPathToA, pos);
    }
}
