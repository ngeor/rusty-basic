use crate::common::{CaseInsensitiveString, Locatable};
use crate::instruction_generator::{AddressOrLabel, Instruction, InstructionNode};
use std::collections::HashMap;

pub struct LabelResolver {
    pub instructions: Vec<InstructionNode>,
}

impl LabelResolver {
    pub fn new(instructions: Vec<InstructionNode>) -> Self {
        Self { instructions }
    }

    pub fn resolve_labels(&mut self) {
        let label_to_address = self.build_label_to_address_map();
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
            Instruction::Jump(AddressOrLabel::Unresolved(x)) => {
                *instruction =
                    Instruction::Jump(AddressOrLabel::Resolved(*label_to_address.get(x).unwrap()));
            }
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(x)) => {
                *instruction = Instruction::JumpIfFalse(AddressOrLabel::Resolved(
                    *label_to_address
                        .get(x)
                        .unwrap_or_else(|| panic!("Label {} not found", x)),
                ));
            }
            Instruction::OnErrorGoTo(AddressOrLabel::Unresolved(x)) => {
                *instruction = Instruction::OnErrorGoTo(AddressOrLabel::Resolved(
                    *label_to_address.get(x).unwrap(),
                ));
            }
            Instruction::GoSub(AddressOrLabel::Unresolved(x)) => {
                *instruction =
                    Instruction::GoSub(AddressOrLabel::Resolved(*label_to_address.get(x).unwrap()));
            }
            Instruction::Return(Some(AddressOrLabel::Unresolved(label))) => {
                *instruction = Instruction::Return(Some(AddressOrLabel::Resolved(
                    *label_to_address.get(label).unwrap(),
                )));
            }
            Instruction::ResumeLabel(AddressOrLabel::Unresolved(label)) => {
                *instruction = Instruction::ResumeLabel(AddressOrLabel::Resolved(
                    *label_to_address.get(label).unwrap(),
                ));
            }
            _ => {}
        }
    }

    fn build_label_to_address_map(&self) -> HashMap<CaseInsensitiveString, usize> {
        let mut result: HashMap<CaseInsensitiveString, usize> = HashMap::new();
        for j in 0..self.instructions.len() {
            if let Instruction::Label(y) = self.instructions[j].as_ref() {
                result.insert(y.clone(), j);
            }
        }
        result
    }
}
