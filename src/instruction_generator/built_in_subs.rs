use super::{Instruction, InstructionGenerator};
use crate::common::Location;
use crate::linter::{BuiltInSub, ExpressionNode};

impl InstructionGenerator {
    pub fn generate_built_in_sub_call_instructions(
        &mut self,
        name: BuiltInSub,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        match name {
            BuiltInSub::System => {
                self.push(Instruction::Halt, pos);
            }
            _ => {
                self.generate_push_unnamed_args_instructions(args, pos);
                self.push(Instruction::PushStack, pos);
                self.push(Instruction::BuiltInSub(name), pos);
                self.push(Instruction::PopStack, pos);
            }
        }
    }
}
