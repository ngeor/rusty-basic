use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{BuiltInFunction, ExpressionNode};

impl InstructionGenerator {
    pub fn generate_built_in_function_call_instructions(
        &mut self,
        function_name: BuiltInFunction,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.generate_push_unnamed_args_instructions(args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInFunction(function_name), pos);

        self.push(Instruction::PopStack, pos);
        self.push(Instruction::CopyResultToA, pos);
    }
}
