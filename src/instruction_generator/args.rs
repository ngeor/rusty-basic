use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{Expression, ExpressionNode, HasExpressionType, ParamName};

impl InstructionGenerator {
    pub fn generate_push_named_args_instructions(
        &mut self,
        param_names: &Vec<ParamName>,
        expressions: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectNamedArguments, pos);
        for (n, e_node) in param_names.iter().zip(expressions.iter()) {
            let Locatable { element: e, pos } = e_node;
            self.generate_expression_instructions_casting(e.clone().at(pos), n.expression_type());
            self.push(Instruction::PushNamed(n.clone()), *pos);
        }
    }

    pub fn generate_push_unnamed_args_instructions(
        &mut self,
        expressions: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectUnnamedArguments, pos);
        for e_node in expressions {
            let Locatable { element: e, pos } = e_node;
            self.generate_expression_instructions(e.clone().at(pos));
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
