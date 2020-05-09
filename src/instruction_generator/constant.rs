use super::{Instruction, InstructionGenerator, Result};
use crate::linter::{ExpressionNode, QNameNode};

impl InstructionGenerator {
    pub fn generate_const_instructions(
        &mut self,
        left: QNameNode,
        right: ExpressionNode,
    ) -> Result<()> {
        let (qualified_name, pos) = left.consume();
        self.generate_expression_instructions(right)?;
        self.push(Instruction::StoreConst(qualified_name), pos);
        Ok(())
    }
}
