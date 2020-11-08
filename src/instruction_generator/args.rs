use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{Expression, ExpressionNode, HasExpressionType, ParamName};
use crate::parser::QualifiedName;

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

    pub fn generate_stash_by_ref_args(&mut self, args: &Vec<ExpressionNode>) {
        let mut idx: usize = 0;
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(_)
                | Expression::Property(_, _, _)
                | Expression::ArrayElement(_, _, _) => {
                    // by ref
                    self.push(Instruction::EnqueueToReturnStack(idx), *pos);
                }
                _ => {}
            }

            idx += 1;
        }
    }

    pub fn generate_un_stash_by_ref_args(&mut self, args: &Vec<ExpressionNode>) {
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(_)
                | Expression::Property(_, _, _)
                | Expression::ArrayElement(_, _, _) => {
                    self.push(Instruction::DequeueFromReturnStack, *pos);
                    self.generate_store_instructions(arg.clone(), *pos);
                }
                _ => {}
            }
        }
    }

    pub fn generate_stash_function_return_value(
        &mut self,
        qualified_name: QualifiedName,
        pos: Location,
    ) {
        self.push(Instruction::StashFunctionReturnValue(qualified_name), pos);
    }

    pub fn generate_un_stash_function_return_value(&mut self, pos: Location) {
        self.push(Instruction::UnStashFunctionReturnValue, pos);
    }
}
