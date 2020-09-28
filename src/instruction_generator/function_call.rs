use crate::common::Locatable;
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::ExpressionNode;
use crate::parser::{BareName, QualifiedNameNode};

impl InstructionGenerator {
    pub fn generate_function_call_instructions(
        &mut self,
        function_name: QualifiedNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let Locatable {
            element: qualified_name,
            pos,
        } = function_name;
        let bare_name: &BareName = qualified_name.as_ref();
        let function_parameters = self.function_context.get(bare_name).unwrap().clone();
        self.generate_push_named_args_instructions(&function_parameters, &args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);
        self.generate_copy_by_ref_to_parent(&args);
        self.push(Instruction::PopStack(Some(qualified_name)), pos);
    }
}
