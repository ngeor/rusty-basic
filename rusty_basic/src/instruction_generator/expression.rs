use rusty_common::*;
use rusty_parser::*;
use rusty_variant::Variant;

use super::{Instruction, InstructionGenerator, RootPath};

impl InstructionGenerator {
    pub fn generate_expression_instructions_casting(
        &mut self,
        expr_pos: ExpressionPos,
        target_type: ExpressionType,
    ) {
        let expression_type = expr_pos.expression_type();
        let pos = expr_pos.pos();
        self.generate_expression_instructions(expr_pos);
        if expression_type != target_type {
            match target_type {
                ExpressionType::BuiltIn(q) => {
                    self.push(Instruction::Cast(q), pos);
                }
                ExpressionType::FixedLengthString(l) => {
                    self.push(Instruction::FixLength(l), pos);
                }
                _ => panic!("Cannot cast {:?} into {:?}", expression_type, target_type),
            }
        }
    }

    pub fn generate_expression_instructions(&mut self, expr_pos: ExpressionPos) {
        self.generate_expression_instructions_optionally_by_ref(expr_pos, true);
    }

    pub fn generate_expression_instructions_optionally_by_ref(
        &mut self,
        expr_pos: ExpressionPos,
        consume_var_path: bool,
    ) {
        let Positioned { element: e, pos } = expr_pos;
        match e {
            Expression::SingleLiteral(s) => {
                self.push_load(Variant::VSingle(s), pos);
            }
            Expression::DoubleLiteral(s) => {
                self.push_load(Variant::VDouble(s), pos);
            }
            Expression::StringLiteral(s) => {
                self.push_load(Variant::VString(s), pos);
            }
            Expression::IntegerLiteral(s) => {
                self.push_load(Variant::VInteger(s), pos);
            }
            Expression::LongLiteral(s) => {
                self.push_load(Variant::VLong(s), pos);
            }
            Expression::Variable(_, _)
            | Expression::ArrayElement(_, _, _)
            | Expression::Property(_, _, _) => {
                self.generate_path_instructions(e.at_pos(pos));
                self.push(Instruction::CopyVarPathToA, pos);
                if consume_var_path {
                    self.push(Instruction::PopVarPath, pos);
                }
            }
            Expression::FunctionCall(n, args) => {
                let name_pos = n.at_pos(pos);
                self.generate_function_call_instructions(name_pos, args);
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
                    Operator::Modulo => self.push(Instruction::Modulo, pos),
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

    pub fn generate_path_instructions(&mut self, expr_pos: ExpressionPos) {
        let Positioned { element: expr, pos } = expr_pos;

        match expr {
            Expression::Variable(var_name, ..) => {
                let linter_var_info: &VariableInfo = self
                    .linter_names
                    .get_resolved_variable_info(&self.current_subprogram, &var_name);

                self.push(
                    Instruction::VarPathName(RootPath {
                        name: var_name,
                        shared: linter_var_info.shared,
                    }),
                    pos,
                );
            }
            Expression::ArrayElement(array_name, indices, ..) => {
                let linter_var_info: &VariableInfo = self
                    .linter_names
                    .get_resolved_variable_info(&self.current_subprogram, &array_name);

                self.push(
                    Instruction::VarPathName(RootPath {
                        name: array_name,
                        shared: linter_var_info.shared,
                    }),
                    pos,
                );
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
                self.generate_path_instructions(left_side.at_pos(pos));
                self.push(
                    Instruction::VarPathProperty(property_name.demand_bare()),
                    pos,
                );
            }
            _ => panic!("Not a name expression {:?}", expr),
        }
    }
}
