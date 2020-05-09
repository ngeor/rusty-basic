use super::{Instruction, InstructionGenerator, Result};
use crate::common::*;
use crate::linter::ConditionalBlockNode;

impl InstructionGenerator {
    pub fn generate_while_instructions(
        &mut self,
        w: ConditionalBlockNode,
        pos: Location,
    ) -> Result<()> {
        let start_idx = self.instructions.len();
        // evaluate condition into register A
        self.generate_expression_instructions(w.condition)?;
        let jump_if_false_idx = self.instructions.len();
        self.push(Instruction::JumpIfFalse(0), pos); // will determine soon
        self.generate_block_instructions(w.statements)?;
        self.push(Instruction::Jump(start_idx), pos);
        let exit_idx = self.instructions.len();
        self.instructions[jump_if_false_idx] = Instruction::JumpIfFalse(exit_idx).at(pos); // patch jump statement with correct index
        Ok(())
    }
}
