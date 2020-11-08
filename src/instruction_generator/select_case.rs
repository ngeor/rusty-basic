use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{CaseExpression, ExpressionNode, SelectCaseNode};
use crate::parser::Operator;

fn next_case_label(case_blocks_len: usize, has_else_block: bool, idx: usize) -> String {
    if idx + 1 < case_blocks_len {
        format!("case{}", idx + 1)
    } else if has_else_block {
        "case-else".to_string()
    } else {
        "end-select".to_string()
    }
}

impl InstructionGenerator {
    pub fn generate_select_case_instructions(&mut self, s: SelectCaseNode, pos: Location) {
        // evaluate SELECT CASE x INTO A
        self.generate_expression_instructions(s.expr);
        self.push(Instruction::PushAToValueStack, pos);
        let mut idx: usize = 0;
        let case_blocks_len = s.case_blocks.len();
        let has_else = s.else_block.is_some();
        for case_block in s.case_blocks {
            self.label(format!("case{}", idx), pos);
            let next_case_label = next_case_label(case_blocks_len, has_else, idx);
            match case_block.expr {
                CaseExpression::Simple(e) => {
                    self.generate_case_expr_simple(e, next_case_label.as_str(), pos);
                }
                CaseExpression::Is(op, e) => {
                    self.generate_case_expr_is(op, e, next_case_label.as_str(), pos);
                }
                CaseExpression::Range(from, to) => {
                    self.generate_case_expr_range(from, to, next_case_label.as_str(), pos);
                }
            }
            self.generate_block_instructions(case_block.statements);
            self.jump("end-select", pos);
            idx += 1;
        }
        match s.else_block {
            Some(e) => {
                self.label("case-else", pos);
                self.generate_block_instructions(e);
            }
            None => {}
        }
        self.push(Instruction::PopValueStackIntoA, pos);
        self.label("end-select", pos);
    }

    fn generate_case_expr_simple(
        &mut self,
        e: ExpressionNode,
        next_case_label: &str,
        pos: Location,
    ) {
        self.generate_expression_instructions(e);
        self.push(Instruction::CopyAToB, pos);
        self.push(Instruction::PopValueStackIntoA, pos);
        self.push(Instruction::PushAToValueStack, pos);
        self.push(Instruction::Equal, pos);
        self.jump_if_false(next_case_label, pos);
    }

    fn generate_case_expr_is(
        &mut self,
        op: Operator,
        e: ExpressionNode,
        next_case_label: &str,
        pos: Location,
    ) {
        self.generate_expression_instructions(e);
        self.push(Instruction::CopyAToB, pos);
        self.push(Instruction::PopValueStackIntoA, pos);
        self.push(Instruction::PushAToValueStack, pos);
        match op {
            Operator::Less => self.push(Instruction::Less, pos),
            Operator::LessOrEqual => self.push(Instruction::LessOrEqual, pos),
            Operator::Greater => self.push(Instruction::Greater, pos),
            Operator::GreaterOrEqual => self.push(Instruction::GreaterOrEqual, pos),
            Operator::Equal => self.push(Instruction::Equal, pos),
            Operator::NotEqual => self.push(Instruction::NotEqual, pos),
            _ => panic!("Unexpected CASE IS operator {:?}", op),
        }
        self.jump_if_false(next_case_label, pos);
    }

    fn generate_case_expr_range(
        &mut self,
        from: ExpressionNode,
        to: ExpressionNode,
        next_case_label: &str,
        pos: Location,
    ) {
        self.generate_expression_instructions(from);
        self.push(Instruction::CopyAToB, pos);
        self.push(Instruction::PopValueStackIntoA, pos);
        self.push(Instruction::PushAToValueStack, pos);
        self.push(Instruction::GreaterOrEqual, pos);
        self.jump_if_false(next_case_label, pos);
        // evaluate to -> A
        self.generate_expression_instructions(to);
        self.push(Instruction::CopyAToB, pos);
        self.push(Instruction::PopValueStackIntoA, pos);
        self.push(Instruction::PushAToValueStack, pos);
        self.push(Instruction::LessOrEqual, pos);
        self.jump_if_false(next_case_label, pos);
    }
}
