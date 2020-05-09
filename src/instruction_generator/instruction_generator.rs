use super::error::Result;
use super::instruction::*;
use crate::common::*;
use crate::linter::*;
use crate::variant::Variant;

use std::collections::HashMap;

// pass 1: collect function names -> parameter names, in order to use them in function/sub calls

type ParamMap = HashMap<CaseInsensitiveString, Vec<QualifiedName>>;

fn collect_parameter_names(program: &ProgramNode) -> (ParamMap, ParamMap) {
    let mut functions: ParamMap = HashMap::new();
    let mut subs: ParamMap = HashMap::new();

    for top_level_token_node in program {
        let top_level_token = top_level_token_node.as_ref();
        match top_level_token {
            TopLevelToken::FunctionImplementation(f) => {
                // collect param names
                functions.insert(
                    f.name.bare_name().clone(),
                    f.params.clone().strip_location(),
                );
            }
            TopLevelToken::SubImplementation(s) => {
                // collect param names
                subs.insert(
                    s.name.bare_name().clone(),
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

pub fn generate_instructions(program: ProgramNode) -> Result<Vec<InstructionNode>> {
    let (f, s) = collect_parameter_names(&program);
    let mut generator = InstructionGenerator::new(f, s);
    generator.generate_unresolved(program)?;
    generator.resolve_instructions();
    Ok(generator.instructions)
}

fn collect_labels(instructions: &Vec<InstructionNode>) -> HashMap<CaseInsensitiveString, usize> {
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

    pub fn generate_unresolved(&mut self, program: ProgramNode) -> Result<()> {
        let mut functions: Vec<(FunctionImplementation, Location)> = vec![];
        let mut subs: Vec<(SubImplementation, Location)> = vec![];

        for t in program {
            let (top_level_token, pos) = t.consume();
            match top_level_token {
                TopLevelToken::Statement(s) => {
                    self.generate_statement_node_instructions(s.at(pos))?;
                }
                TopLevelToken::FunctionImplementation(f) => functions.push((f, pos)),
                TopLevelToken::SubImplementation(s) => subs.push((s, pos)),
            }
        }

        // add HALT instruction at end of program to separate from the functions and subs
        self.push(
            Instruction::Halt,
            Location::new(std::u32::MAX, std::u32::MAX),
        );

        // functions
        for (f, pos) in functions {
            let name = f.name;
            let block = f.body;
            self.function_label(name.bare_name(), pos);
            // set default value
            self.push(
                Instruction::Load(Variant::default_variant(name.qualifier())),
                pos,
            );
            self.push(Instruction::StoreAToResult, pos);
            self.generate_block_instructions(block)?;
            self.push(Instruction::PopRet, pos);
        }

        // subs
        for (s, pos) in subs {
            let name = s.name;
            let block = s.body;
            self.sub_label(name.bare_name(), pos);
            self.generate_block_instructions(block)?;
            self.push(Instruction::PopRet, pos);
        }

        Ok(())
    }

    pub fn resolve_instructions(&mut self) {
        let labels = collect_labels(&self.instructions);
        // resolve jumps
        for instruction_node in self.instructions.iter_mut() {
            let instruction: &Instruction = instruction_node.as_ref();
            let pos: Location = instruction_node.location();
            if let Instruction::UnresolvedJump(x) = instruction {
                *instruction_node = Instruction::Jump(*labels.get(x).unwrap()).at(pos);
            } else if let Instruction::UnresolvedJumpIfFalse(x) = instruction {
                *instruction_node = Instruction::JumpIfFalse(*labels.get(x).unwrap()).at(pos);
            } else if let Instruction::SetUnresolvedErrorHandler(x) = instruction {
                *instruction_node = Instruction::SetErrorHandler(*labels.get(x).unwrap()).at(pos);
            }
        }
    }

    pub fn push(&mut self, i: Instruction, pos: Location) {
        self.instructions.push(i.at(pos));
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
        l: QNameNode,
        r: ExpressionNode,
    ) -> Result<()> {
        self.generate_expression_instructions(r)?;
        let pos = l.location();
        self.push(Instruction::Store(l.strip_location()), pos);
        Ok(())
    }
}
