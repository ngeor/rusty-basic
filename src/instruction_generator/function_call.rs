use super::instruction::*;
use super::InstructionGenerator;
use crate::common::*;
use crate::linter::*;

impl InstructionGenerator {
    pub fn generate_function_call_instructions(
        &mut self,
        function_name: QNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let pos = function_name.location();

        let bare_name: &CaseInsensitiveString = function_name.bare_name();
        let function_parameters = self.function_context.get(bare_name).unwrap().clone();
        self.generate_push_named_args_instructions(function_parameters, args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);

        self.push(Instruction::PopStack, pos);
        self.push(Instruction::CopyResultToA, pos);
    }

    pub fn generate_push_named_args_instructions(
        &mut self,
        param_names: Vec<QualifiedName>,
        expressions: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::PreparePush, pos);
        for (n, e_node) in param_names.into_iter().zip(expressions.into_iter()) {
            let (e, pos) = e_node.consume();
            match e {
                Expression::Variable(v_name) => {
                    self.push(
                        Instruction::SetNamedRefParam(NamedRefParam {
                            parameter_name: n,
                            argument_name: v_name,
                        }),
                        pos,
                    );
                }
                _ => {
                    self.generate_expression_instructions(e.at(pos));
                    self.push(Instruction::SetNamedValParam(n), pos);
                }
            }
        }
    }

    pub fn generate_push_unnamed_args_instructions(
        &mut self,
        expressions: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::PreparePush, pos);
        for e_node in expressions.into_iter() {
            let (e, pos) = e_node.consume();
            match e {
                Expression::Variable(v_name) => {
                    self.push(Instruction::PushUnnamedRefParam(v_name), pos);
                }
                _ => {
                    self.generate_expression_instructions(e.at(pos));
                    self.push(Instruction::PushUnnamedValParam, pos);
                }
            }
        }
    }
}
