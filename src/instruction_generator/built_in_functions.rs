use super::{Instruction, InstructionGenerator};
use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::{
    Expression, ExpressionNode, ExpressionType, HasExpressionType, ParamName, ParamType,
};
use crate::parser::{BareName, TypeQualifier};
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
        self.generate_copy_by_ref_to_parent_unnamed(&args);
        self.push(Instruction::PopStack(Some(function_name.into())), pos);
    }

    /// Generates the instructions that copy back ByRef parameters to the parent context.
    pub fn generate_copy_by_ref_to_parent_unnamed(&mut self, args: &Vec<ExpressionNode>) {
        let mut idx: u8 = 0;
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(arg_name) => {
                    // by ref
                    // copy to parent context
                    let dummy_name: BareName = format!("{}", idx).into();
                    let param_type: ParamType = match arg.expression_type() {
                        ExpressionType::BuiltIn(q) => ParamType::BuiltIn(q),
                        ExpressionType::UserDefined(u) => ParamType::UserDefined(u),
                        ExpressionType::FixedLengthString(_) => {
                            ParamType::BuiltIn(TypeQualifier::DollarString)
                        }
                        ExpressionType::FileHandle => {
                            panic!("file handle variables should not be supported")
                        }
                    };
                    self.push(
                        Instruction::CopyToParent(
                            ParamName::new(dummy_name, param_type),
                            arg_name.clone(),
                        ),
                        *pos,
                    );
                }
                _ => {}
            }

            idx += 1;
        }
    }
}
