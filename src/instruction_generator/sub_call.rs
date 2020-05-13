use super::{Instruction, InstructionGenerator};
use crate::linter::{BareNameNode, ExpressionNode};

impl InstructionGenerator {
    pub fn generate_sub_call_instructions(
        &mut self,
        name_node: BareNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let (name, pos) = name_node.consume();
        let sub_impl_parameters = self.sub_context.get(&name).unwrap().clone();
        self.generate_push_named_args_instructions(sub_impl_parameters, args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_sub(name, pos);
        self.push(Instruction::PopStack, pos);
    }
}
