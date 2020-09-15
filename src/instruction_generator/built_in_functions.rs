use super::{Instruction, InstructionGenerator};
use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::ExpressionNode;
use crate::parser::{HasQualifier, QualifiedName};

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
        self.push(
            Instruction::PopStack(Some(QualifiedName::new(
                function_name.into(),
                function_name.qualifier(),
            ))),
            pos,
        );
    }
}
