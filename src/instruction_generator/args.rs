use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{Expression, ExpressionNode, HasExpressionType, ParamName};

impl InstructionGenerator {
    pub fn generate_push_named_args_instructions(
        &mut self,
        param_names: &Vec<ParamName>,
        args: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectArguments, pos);
        for (param_name, Locatable { element: arg, pos }) in param_names.iter().zip(args.iter()) {
            self.generate_expression_instructions_casting(
                arg.clone().at(pos),
                param_name.expression_type(),
            );
            self.push(Instruction::PushNamed(param_name.clone()), *pos);
        }
    }

    pub fn generate_push_unnamed_args_instructions(
        &mut self,
        args: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectArguments, pos);
        for Locatable { element: arg, pos } in args {
            self.generate_expression_instructions(arg.clone().at(pos));
            self.push(Instruction::PushUnnamed, *pos);
        }
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
