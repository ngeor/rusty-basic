use super::instruction::*;
use crate::common::*;
use crate::linter::*;
use crate::parser::{BareName, HasQualifier};
use crate::variant::Variant;
use std::collections::HashMap;

// pass 1: collect function/sub names -> parameter names, in order to use them in function/sub calls

type ParamMap = HashMap<CaseInsensitiveString, Vec<ResolvedParamName>>;

fn collect_parameter_names(program: &ProgramNode) -> (ParamMap, ParamMap) {
    let mut functions: ParamMap = HashMap::new();
    let mut subs: ParamMap = HashMap::new();

    for top_level_token_node in program {
        let top_level_token = top_level_token_node.as_ref();
        match top_level_token {
            TopLevelToken::FunctionImplementation(f) => {
                // collect param names
                functions.insert((&f.name).into(), f.params.clone().strip_location());
            }
            TopLevelToken::SubImplementation(s) => {
                // collect param names
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
    generator.resolve_instructions();
    generator.instructions
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
            let bare_name: &BareName = name.as_ref();
            let block = f.body;
            self.function_label(bare_name, pos);
            // set default value
            self.push(Instruction::Load(Variant::from(name.qualifier())), pos);
            self.generate_block_instructions(block);
            self.push(Instruction::PopRet, pos);
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

    pub fn resolve_instructions(&mut self) {
        let labels = collect_labels(&self.instructions);
        // resolve jumps
        for instruction_node in self.instructions.iter_mut() {
            let instruction: &Instruction = instruction_node.as_ref();
            let pos: Location = instruction_node.pos();
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
        l: DimName,
        r: ExpressionNode,
        pos: Location,
    ) {
        let left_type = l.type_definition();
        self.generate_expression_instructions_casting(r, left_type);
        self.push(Instruction::Store(l), pos);
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{AtRowCol, StripLocation};
    use crate::instruction_generator::test_utils::*;
    use crate::instruction_generator::Instruction;
    use crate::linter::DimName;
    use crate::parser::TypeQualifier;
    use crate::variant::Variant;

    #[test]
    fn test_assignment() {
        assert_eq!(
            generate_instructions_str("X = 1"),
            [
                Instruction::Load(Variant::VInteger(1)).at_rc(1, 5),
                Instruction::Cast(TypeQualifier::BangSingle).at_rc(1, 5),
                Instruction::Store(DimName::parse("X!")).at_rc(1, 1),
                Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
            ]
        );
    }

    #[test]
    fn test_assignment_no_cast() {
        assert_eq!(
            generate_instructions_str("X% = 1").strip_location(),
            [
                Instruction::Load(Variant::VInteger(1)),
                Instruction::Store(DimName::parse("X%")),
                Instruction::Halt
            ]
        );
    }

    #[test]
    fn test_assignment_binary_plus() {
        assert_eq!(
            generate_instructions_str("X% = 1 + 2.1").strip_location(),
            [
                Instruction::PushRegisters,
                Instruction::Load(Variant::VInteger(1)),
                Instruction::CopyAToB,
                Instruction::Load(Variant::VSingle(2.1)),
                Instruction::SwapAWithB,
                Instruction::Plus,
                Instruction::PopRegisters,
                Instruction::Cast(TypeQualifier::PercentInteger),
                Instruction::Store(DimName::parse("X%")),
                Instruction::Halt
            ]
        );
    }
}
