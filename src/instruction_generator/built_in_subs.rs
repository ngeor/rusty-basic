use super::{Instruction, InstructionGenerator, Result};
use crate::linter::{BareNameNode, ExpressionNode};

impl InstructionGenerator {
    pub fn generate_built_in_sub_call_instructions(
        &mut self,
        name_node: BareNameNode,
        args: Vec<ExpressionNode>,
    ) -> Result<()> {
        let (name, pos) = name_node.consume();
        if &name == "SYSTEM" {
            self.push(Instruction::Halt, pos);
        } else {
            self.generate_push_unnamed_args_instructions(args, pos)?;
            self.push(Instruction::PushStack, pos);
            self.push(Instruction::BuiltInSub(name), pos);
        }
        Ok(())
    }
}
