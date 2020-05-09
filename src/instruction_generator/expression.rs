use super::{Instruction, InstructionGenerator, Result};
use crate::common::*;
use crate::linter::*;
use crate::variant::Variant;

impl InstructionGenerator {
    pub fn generate_expression_instructions(&mut self, expr_node: ExpressionNode) -> Result<()> {
        let (e, pos) = expr_node.consume();
        match e {
            Expression::SingleLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
                Ok(())
            }
            Expression::DoubleLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
                Ok(())
            }
            Expression::StringLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
                Ok(())
            }
            Expression::IntegerLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
                Ok(())
            }
            Expression::LongLiteral(s) => {
                self.push(Instruction::Load(Variant::from(s)), pos);
                Ok(())
            }
            Expression::Variable(name) => {
                self.push(Instruction::CopyVarToA(name), pos);
                Ok(())
            }
            Expression::Constant(name) => {
                self.push(Instruction::CopyVarToA(name), pos);
                Ok(())
            }
            Expression::FunctionCall(n, args) => {
                let name_node = n.at(pos);
                self.generate_function_call_instructions(name_node, args)
            }
            Expression::BinaryExpression(op, left, right) => {
                self.push(Instruction::PushRegisters, pos);
                self.generate_expression_instructions(*left)?;
                self.push(Instruction::CopyAToB, pos);
                self.generate_expression_instructions(*right)?;
                self.push(Instruction::SwapAWithB, pos);

                match op {
                    Operand::Plus => self.push(Instruction::Plus, pos),
                    Operand::Minus => self.push(Instruction::Minus, pos),
                    Operand::LessThan => self.push(Instruction::LessThan, pos),
                    Operand::LessOrEqualThan => self.push(Instruction::LessOrEqualThan, pos),
                }
                self.push(Instruction::PopRegisters, pos);
                Ok(())
            }
            Expression::UnaryExpression(op, child) => {
                match op {
                    UnaryOperand::Not => {
                        self.generate_expression_instructions(*child)?;
                        self.push(Instruction::NotA, pos);
                    }
                    UnaryOperand::Minus => {
                        self.generate_expression_instructions(*child)?;
                        self.push(Instruction::NegateA, pos);
                    }
                }
                Ok(())
            }
        }
    }
}
