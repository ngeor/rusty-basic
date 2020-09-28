use crate::built_ins::BuiltInFunction;
use crate::common::Location;
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::ExpressionNode;

impl InstructionGenerator {
    pub fn generate_built_in_function_call_instructions(
        &mut self,
        function_name: BuiltInFunction,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.generate_push_unnamed_args_instructions(&args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInFunction(function_name), pos);
        self.generate_copy_by_ref_to_parent(&args);
        self.push(Instruction::PopStack(Some(function_name.into())), pos);
    }
}
