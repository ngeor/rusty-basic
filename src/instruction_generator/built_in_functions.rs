use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{ExpressionNode, QNameNode};

impl InstructionGenerator {
    pub fn generate_built_in_function_call_instructions(
        &mut self,
        function_name: QNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let pos = function_name.location();
        self.generate_push_unnamed_args_instructions(args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(
            Instruction::BuiltInFunction(function_name.strip_location()),
            pos,
        );
    }
}
