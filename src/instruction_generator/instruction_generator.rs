use super::instruction::*;
use crate::common::*;
use crate::parser::{
    BareName, Expression, ExpressionNode, FunctionImplementation, HasExpressionType, Name,
    ParamName, ProgramNode, QualifiedName, SubImplementation, TopLevelToken,
};
use crate::variant::Variant;
use std::collections::HashMap;

// pass 1: collect function/sub names -> parameter names, in order to use them in function/sub calls

type ParamMap = HashMap<CaseInsensitiveString, Vec<ParamName>>;

fn collect_parameter_names(program: &ProgramNode) -> (ParamMap, ParamMap) {
    let mut functions: ParamMap = HashMap::new();
    let mut subs: ParamMap = HashMap::new();

    for top_level_token_node in program {
        let top_level_token = top_level_token_node.as_ref();
        match top_level_token {
            TopLevelToken::FunctionImplementation(f) => {
                let FunctionImplementation {
                    name: Locatable { element: name, .. },
                    params,
                    ..
                } = f;
                let bare_name: &BareName = name.bare_name();
                functions.insert(
                    bare_name.clone(),
                    params.iter().map(|p| p.as_ref().clone()).collect(),
                );
            }
            TopLevelToken::SubImplementation(s) => {
                subs.insert(
                    s.name.clone().strip_location(),
                    s.params.clone().strip_location(),
                );
            }
            _ => (),
        }
    }

    (functions, subs)
}

pub struct InstructionGenerator {
    pub instructions: Vec<InstructionNode>,
    pub function_context: ParamMap,
    pub sub_context: ParamMap,
}

pub fn generate_instructions(program: ProgramNode) -> Vec<InstructionNode> {
    let (f, s) = collect_parameter_names(&program);
    let mut generator = InstructionGenerator::new(f, s);
    generator.generate_unresolved(program);
    generator.resolve_labels();
    generator.instructions
}

fn build_label_to_address_map(
    instructions: &Vec<InstructionNode>,
) -> HashMap<CaseInsensitiveString, usize> {
    let mut result: HashMap<CaseInsensitiveString, usize> = HashMap::new();
    for j in 0..instructions.len() {
        if let Instruction::Label(y) = instructions[j].as_ref() {
            result.insert(y.clone(), j);
        }
    }
    result
}

impl InstructionGenerator {
    pub fn new(function_context: ParamMap, sub_context: ParamMap) -> Self {
        Self {
            instructions: vec![],
            function_context,
            sub_context,
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
            self.push(Instruction::PopRet, pos);
        }
    }

    pub fn resolve_labels(&mut self) {
        let label_to_address = build_label_to_address_map(&self.instructions);
        for instruction_node in self.instructions.iter_mut() {
            let Locatable {
                element: instruction,
                ..
            } = instruction_node;
            Self::resolve_label(instruction, &label_to_address);
        }
    }

    fn resolve_label(
        instruction: &mut Instruction,
        label_to_address: &HashMap<CaseInsensitiveString, usize>,
    ) {
        match instruction {
            Instruction::UnresolvedJump(x) => {
                *instruction = Instruction::Jump(*label_to_address.get(x).unwrap());
            }
            Instruction::UnresolvedJumpIfFalse(x) => {
                *instruction = Instruction::JumpIfFalse(
                    *label_to_address
                        .get(x)
                        .expect(&format!("Label {} not found", x)),
                );
            }
            Instruction::SetUnresolvedErrorHandler(x) => {
                *instruction = Instruction::SetErrorHandler(*label_to_address.get(x).unwrap());
            }
            Instruction::UnresolvedGoSub(x) => {
                *instruction = Instruction::GoSub(*label_to_address.get(x).unwrap());
            }
            Instruction::UnresolvedReturn(opt_label) => match opt_label {
                Some(label) => {
                    *instruction = Instruction::Return(Some(*label_to_address.get(label).unwrap()));
                }
                _ => {
                    *instruction = Instruction::Return(None);
                }
            },
            Instruction::UnresolvedResumeLabel(label) => {
                *instruction = Instruction::ResumeLabel(*label_to_address.get(label).unwrap());
            }
            _ => {}
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
            Instruction::UnresolvedJumpIfFalse(CaseInsensitiveString::new(format!(
                "_{}_{:?}",
                prefix.as_ref(),
                pos
            ))),
            pos,
        );
    }

    pub fn jump<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::UnresolvedJump(CaseInsensitiveString::new(format!(
                "_{}_{:?}",
                prefix.as_ref(),
                pos
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
            Instruction::UnresolvedJump(CaseInsensitiveString::new(format!(
                ":fun:{}",
                name.as_ref(),
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
            Instruction::UnresolvedJump(CaseInsensitiveString::new(format!(
                ":sub:{}",
                name.as_ref(),
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
