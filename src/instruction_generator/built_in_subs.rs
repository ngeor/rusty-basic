use crate::built_ins::BuiltInSub;
use crate::common::Location;
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::parser::ExpressionNode;

impl InstructionGenerator {
    pub fn generate_built_in_sub_call_instructions(
        &mut self,
        name: BuiltInSub,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.generate_push_unnamed_args_instructions(&args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInSub(name), pos);
        self.generate_stash_by_ref_args(&args);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
    }
}
