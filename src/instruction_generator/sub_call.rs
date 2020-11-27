use crate::common::Locatable;
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::parser::{BareNameNode, ExpressionNode};

impl InstructionGenerator {
    pub fn generate_sub_call_instructions(
        &mut self,
        name_node: BareNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let Locatable { element: name, pos } = name_node;
        let sub_impl_parameters = self.sub_context.get(&name).unwrap().clone();
        self.generate_push_named_args_instructions(&sub_impl_parameters, &args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_sub(name, pos);
        self.generate_stash_by_ref_args(&args);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
    }
}
