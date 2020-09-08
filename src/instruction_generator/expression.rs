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
                let x = ResolvedDeclaredName {
                    name,
                    type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
                };
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
                    Operand::Plus => self.push(Instruction::Plus, pos),
                    Operand::Minus => self.push(Instruction::Minus, pos),
                    Operand::Multiply => self.push(Instruction::Multiply, pos),
                    Operand::Divide => self.push(Instruction::Divide, pos),
                    Operand::Less => self.push(Instruction::Less, pos),
                    Operand::LessOrEqual => self.push(Instruction::LessOrEqual, pos),
                    Operand::Equal => self.push(Instruction::Equal, pos),
                    Operand::GreaterOrEqual => self.push(Instruction::GreaterOrEqual, pos),
                    Operand::Greater => self.push(Instruction::Greater, pos),
                    Operand::NotEqual => self.push(Instruction::NotEqual, pos),
                    Operand::And => self.push(Instruction::And, pos),
                    Operand::Or => self.push(Instruction::Or, pos),
                }
                self.push(Instruction::PopRegisters, pos);
            }
            Expression::UnaryExpression(op, child) => match op {
                UnaryOperand::Not => {
                    self.generate_expression_instructions(*child);
                    self.push(Instruction::NotA, pos);
                }
                UnaryOperand::Minus => {
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
