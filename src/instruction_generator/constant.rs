use super::{Instruction, InstructionGenerator};
use crate::common::Locatable;
use crate::parser::QualifiedNameNode;
use crate::variant::Variant;

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QualifiedNameNode, right: Variant) {
        let Locatable {
            element: qualified_name,
            pos,
        } = left;
        self.push(Instruction::Load(right), pos);
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
