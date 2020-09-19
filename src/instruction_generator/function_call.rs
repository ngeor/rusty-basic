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
        self.generate_push_named_args_instructions(function_parameters, args, pos);
        self.push(Instruction::PushStack, pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);
        self.push(Instruction::PopStack(Some(qualified_name)), pos);
    }

    pub fn generate_push_named_args_instructions(
        &mut self,
        param_names: Vec<ResolvedParamName>,
        expressions: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectNamedArguments, pos);
        for (n, e_node) in param_names.into_iter().zip(expressions.into_iter()) {
            let Locatable { element: e, pos } = e_node;
            match e {
                Expression::Variable(v_name) => {
                    self.push(Instruction::PushNamedRef(n, v_name), pos);
                }
                _ => {
                    self.generate_expression_instructions_casting(e.at(pos), n.type_definition());
                    self.push(Instruction::PushNamedVal(n), pos);
                }
            }
        }
    }

    pub fn generate_push_unnamed_args_instructions(
        &mut self,
        expressions: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectUnnamedArguments, pos);
        for e_node in expressions.into_iter() {
            let Locatable { element: e, pos } = e_node;
            match e {
                Expression::Variable(v_name) => {
                    self.push(Instruction::PushUnnamedRef(v_name), pos);
                }
                _ => {
                    self.generate_expression_instructions(e.at(pos));
                    self.push(Instruction::PushUnnamedVal, pos);
                }
            }
        }
    }
}
