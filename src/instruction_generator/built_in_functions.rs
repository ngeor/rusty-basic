use super::{Instruction, InstructionGenerator};
use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::{Expression, ExpressionNode};

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

    /// Generates the instructions that copy back ByRef parameters to the parent context.
    pub fn generate_copy_by_ref_to_parent(&mut self, args: &Vec<ExpressionNode>) {
        let mut idx: usize = 0;
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(arg_name) => {
                    // by ref
                    self.push(Instruction::CopyToParent(idx, arg_name.clone()), *pos);
                }
                _ => {}
            }

            idx += 1;
        }
    }
}
