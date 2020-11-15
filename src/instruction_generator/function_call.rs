use crate::common::Locatable;
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::parser::{BareName, ExpressionNode, NameNode};

impl InstructionGenerator {
    pub fn generate_function_call_instructions(
        &mut self,
        function_name: NameNode,
        args: Vec<ExpressionNode>,
    ) {
        let Locatable { element: name, pos } = function_name;
        let qualified_name = name.demand_qualified();
        let bare_name: &BareName = qualified_name.as_ref();
        let function_parameters = self.function_context.get(bare_name).unwrap().clone();
        self.generate_push_named_args_instructions(&function_parameters, &args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);
        // stash by-ref variables
        self.generate_stash_by_ref_args(&args);
        // stash function name
        self.generate_stash_function_return_value(qualified_name, pos);
        // switch to parent context
        self.push(Instruction::PopStack, pos);
        // un-stash by-ref variables
        self.generate_un_stash_by_ref_args(&args);
        // un-stash function name
        self.generate_un_stash_function_return_value(pos);
    }
}
