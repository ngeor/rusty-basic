use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::*;
use crate::variant::Variant;

impl InstructionGenerator {
    pub fn generate_expression_instructions(&mut self, expr_node: ExpressionNode) {
        let Locatable { element: e, pos } = expr_node;
        match e {
            Expression::SingleLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
            }
            Expression::DoubleLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
            }
            Expression::StringLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
            }
            Expression::IntegerLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
            }
            Expression::LongLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
            }
            Expression::Variable(name) => {
                self.push(Instruction::CopyVarToA(name), pos);
            }
            Expression::Constant(name) => {
                let QualifiedName { name, qualifier } = name;
                let x =
                    ResolvedDeclaredName::single(name, ResolvedTypeDefinition::BuiltIn(qualifier));
                self.push(Instruction::CopyVarToA(x), pos);
            }
            Expression::FunctionCall(n, args) => {
                let name_node = n.at(pos);
                self.generate_function_call_instructions(name_node, args);
            }
            Expression::BuiltInFunctionCall(n, args) => {
                self.generate_built_in_function_call_instructions(n, args, pos);
            }
            Expression::BinaryExpression(op, left, right) => {
                self.push(Instruction::PushRegisters, pos);
                self.generate_expression_instructions(*left);
                self.push(Instruction::CopyAToB, pos);
                self.generate_expression_instructions(*right);
                self.push(Instruction::SwapAWithB, pos);
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
                self.push(Instruction::PopRegisters, pos);
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
            Expression::FileHandle(i) => {
                self.push(Instruction::Load(Variant::VFileHandle(i)), pos);
            }
        }
    }
}
