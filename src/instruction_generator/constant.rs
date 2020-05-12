use super::{Instruction, InstructionGenerator};
use crate::linter::{ExpressionNode, QNameNode};

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QNameNode, right: ExpressionNode) {
        let (qualified_name, pos) = left.consume();
        self.generate_expression_instructions(right);
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
