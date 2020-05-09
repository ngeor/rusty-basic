use super::{InstructionGenerator, Result};
use crate::common::*;
use crate::linter::IfBlockNode;

impl InstructionGenerator {
    pub fn generate_if_block_instructions(
        &mut self,
        if_block_statement: IfBlockNode,
        pos: Location,
    ) -> Result<()> {
        let IfBlockNode {
            if_block,
            else_if_blocks,
            else_block,
        } = if_block_statement;

        // evaluate condition into A
        self.generate_expression_instructions(if_block.condition)?;

        // if false, jump to next one (first else-if or else or end-if)
        let next_label = if else_if_blocks.len() > 0 {
            "else-if-0"
        } else if else_block.is_some() {
            "else"
        } else {
            "end-if"
        };
        self.jump_if_false(next_label, pos);

        // if true, run statements and jump out
        self.generate_block_instructions(if_block.statements)?;
        self.jump("end-if", pos);

        for i in 0..else_if_blocks.len() {
            let else_if_block = else_if_blocks[i].clone();
            self.label(format!("else-if-{}", i), pos);

            // evaluate condition into A
            self.generate_expression_instructions(else_if_block.condition)?;

            // if false, jump to next one (next else-if or else or end-if)
            let next_label = if i + 1 < else_if_blocks.len() {
                format!("else-if-{}", i + 1)
            } else if else_block.is_some() {
                format!("else")
            } else {
                format!("end-if")
            };
            self.jump_if_false(next_label, pos);

            // if true, run statements and jump out
            self.generate_block_instructions(else_if_block.statements)?;
            self.jump("end-if", pos);
        }

        match else_block {
            Some(e) => {
                self.label("else", pos);
                self.generate_block_instructions(e)?;
            }
            None => (),
        }
        self.label("end-if", pos);
        Ok(())
    }
}
