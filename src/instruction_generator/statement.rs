use super::{Instruction, InstructionGenerator, Result};
use crate::common::*;
use crate::linter::{Statement, StatementNode, StatementNodes};

impl InstructionGenerator {
    pub fn generate_block_instructions(&mut self, block: StatementNodes) -> Result<()> {
        for s in block {
            self.generate_statement_node_instructions(s)?;
        }
        Ok(())
    }

    pub fn generate_statement_node_instructions(
        &mut self,
        statement_node: StatementNode,
    ) -> Result<()> {
        let (statement, pos) = statement_node.consume();
        match statement {
            Statement::Assignment(left_side, right_side) => {
                self.generate_assignment_instructions(left_side.at(pos), right_side)
            }
            Statement::SubCall(n, args) => self.generate_sub_call_instructions(n.at(pos), args),
            Statement::ForLoop(f) => self.generate_for_loop_instructions(f, pos),
            Statement::IfBlock(i) => self.generate_if_block_instructions(i, pos),
            Statement::While(w) => self.generate_while_instructions(w, pos),
            Statement::Const(n, e) => self.generate_const_instructions(n, e),
            Statement::ErrorHandler(label) => {
                self.push(Instruction::SetUnresolvedErrorHandler(label), pos);
                Ok(())
            }
            Statement::Label(name) => {
                self.push(Instruction::Label(name.clone()), pos);
                Ok(())
            }
            Statement::GoTo(name) => {
                self.push(Instruction::UnresolvedJump(name.clone()), pos);
                Ok(())
            }
            Statement::SetReturnValue(e) => {
                self.generate_expression_instructions(e)?;
                self.push(Instruction::StoreAToResult, pos);
                Ok(())
            }
        }
    }
}
