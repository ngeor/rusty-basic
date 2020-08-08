use super::{Instruction, InstructionGenerator};
use crate::common::Locatable;
use crate::linter::{ExpressionNode, QNameNode};

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QNameNode, right: ExpressionNode) {
        let Locatable {
            element: qualified_name,
            pos,
        } = left;
        self.generate_expression_instructions(right);
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
