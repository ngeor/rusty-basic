use super::instruction::*;
use super::InstructionGenerator;
use crate::common::*;
use crate::linter::*;
use crate::parser::QualifiedNameNode;

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
        let bare_name: &CaseInsensitiveString = qualified_name.as_ref();
        let function_parameters = self.function_context.get(bare_name).unwrap().clone();
        self.generate_push_named_args_instructions(&function_parameters, &args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);
        self.generate_copy_by_ref_to_parent_named(&function_parameters, &args);
        self.push(Instruction::PopStack(Some(qualified_name)), pos);
    }

    /// Generates the instructions that copy back ByRef parameters to the parent context.
    pub fn generate_copy_by_ref_to_parent_named(
        &mut self,
        param_names: &Vec<ParamName>,
        args: &Vec<ExpressionNode>,
    ) {
        for (param_name, Locatable { element: arg, pos }) in param_names.iter().zip(args.iter()) {
            match arg {
                Expression::Variable(arg_name) => {
                    // by ref
                    // copy to parent context
                    self.push(
                        Instruction::CopyToParent(param_name.clone(), arg_name.clone()),
                        *pos,
                    );
                }
                _ => {}
            }
        }
    }

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
}
