use super::{Instruction, InstructionGenerator};
use crate::common::Locatable;
use crate::linter::{ExpressionNode, QualifiedNameNode};

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QualifiedNameNode, right: ExpressionNode) {
        let Locatable {
            element: qualified_name,
            pos,
        } = left;
        self.generate_expression_instructions(right);
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
