use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::*;
use crate::parser::{Operator, TypeQualifier, UnaryOperator};

impl InstructionGenerator {
    pub fn generate_expression_instructions_casting(
        &mut self,
        expr_node: ExpressionNode,
        target_type: ExpressionType,
    ) {
        let expression_type = expr_node.expression_type();
        let pos = expr_node.pos();
        self.generate_expression_instructions(expr_node);
        if expression_type != target_type {
            match target_type {
                ExpressionType::BuiltIn(q) => {
                    self.push(Instruction::Cast(q), pos);
                }
                ExpressionType::FixedLengthString(l) => {
                    self.push(Instruction::FixLength(l), pos);
                }
                _ => panic!("Cannot cast user defined type"),
            }
        }
    }

    pub fn generate_expression_instructions(&mut self, expr_node: ExpressionNode) {
        let Locatable { element: e, pos } = expr_node;
        match e {
            Expression::SingleLiteral(s) => {
                self.push_load(s, pos);
            }
            Expression::DoubleLiteral(s) => {
                self.push_load(s, pos);
            }
            Expression::StringLiteral(s) => {
                self.push_load(s, pos);
            }
            Expression::IntegerLiteral(s) => {
                self.push_load(s, pos);
            }
            Expression::LongLiteral(s) => {
                self.push_load(s, pos);
            }
            Expression::Variable(_, _)
            | Expression::Constant(_)
            | Expression::ArrayElement(_, _, _)
            | Expression::Property(_, _, _) => self.generate_load_instructions(e, pos),
            Expression::FunctionCall(n, args) => {
                let name_node = n.at(pos);
                self.generate_function_call_instructions(name_node, args);
            }
            Expression::BuiltInFunctionCall(n, args) => {
                self.generate_built_in_function_call_instructions(n, args, pos);
            }
            Expression::BinaryExpression(op, left, right, _) => {
                self.generate_expression_instructions(*left);
                self.push(Instruction::PushAToValueStack, pos);
                self.generate_expression_instructions(*right);
                self.push(Instruction::CopyAToB, pos);
                self.push(Instruction::PopValueStackIntoA, pos);
                match op {
                    Operator::Plus => self.push(Instruction::Plus, pos),
                    Operator::Minus => self.push(Instruction::Minus, pos),
                    Operator::Multiply => self.push(Instruction::Multiply, pos),
                    Operator::Divide => self.push(Instruction::Divide, pos),
                    Operator::Less => self.push(Instruction::Less, pos),
                    Operator::LessOrEqual => self.push(Instruction::LessOrEqual, pos),
                    Operator::Equal => self.push(Instruction::Equal, pos),
                    Operator::GreaterOrEqual => self.push(Instruction::GreaterOrEqual, pos),
                    Operator::Greater => self.push(Instruction::Greater, pos),
                    Operator::NotEqual => self.push(Instruction::NotEqual, pos),
                    Operator::And => self.push(Instruction::And, pos),
                    Operator::Or => self.push(Instruction::Or, pos),
                }
            }
            Expression::UnaryExpression(op, child) => match op {
                UnaryOperator::Not => {
                    self.generate_expression_instructions(*child);
                    self.push(Instruction::NotA, pos);
                }
                UnaryOperator::Minus => {
                    self.generate_expression_instructions(*child);
                    self.push(Instruction::NegateA, pos);
                }
            },
            Expression::Parenthesis(child) => {
                self.generate_expression_instructions(*child);
            }
        }
    }

    pub fn generate_path_instructions(&mut self, expr_node: ExpressionNode) {
        let Locatable { element: expr, pos } = expr_node;

        match expr {
            Expression::Variable(var_name, _) => {
                self.push(Instruction::VarPathName(var_name), pos);
            }
            Expression::ArrayElement(array_name, indices, _element_type) => {
                self.push(Instruction::VarPathName(array_name), pos);
                for arg in indices {
                    let arg_pos = arg.pos();
                    self.push(Instruction::PushAToValueStack, arg_pos);
                    self.generate_expression_instructions_casting(
                        arg,
                        ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
                    );
                    self.push(Instruction::VarPathIndex, arg_pos);
                    self.push(Instruction::PopValueStackIntoA, arg_pos);
                }
            }
            Expression::Property(box_left_side, property_name, _element_type) => {
                let left_side = *box_left_side;
                self.generate_path_instructions(left_side.at(pos));
                self.push(Instruction::VarPathProperty(property_name), pos);
            }
            _ => panic!("Not a name expression"),
        }
    }
}
