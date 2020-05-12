use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{CaseExpression, Operand, SelectCaseNode};

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
        // copy A to B
        self.push(Instruction::CopyAToB, pos);
        let mut idx: usize = 0;
        let case_blocks_len = s.case_blocks.len();
        let has_else = s.else_block.is_some();
        for case_block in s.case_blocks {
            self.label(format!("case{}", idx), pos);
            match case_block.expr {
                CaseExpression::Simple(e) => {
                    // evaluate CASE y -> A
                    self.generate_expression_instructions(e);
                    self.push(Instruction::Equal, pos);
                    self.jump_if_false(next_case_label(case_blocks_len, has_else, idx), pos);
                    self.generate_block_instructions(case_block.statements);
                    self.jump("end-select", pos);
                }
                CaseExpression::Is(op, e) => {
                    // evaluate CASE IS y -> A
                    self.generate_expression_instructions(e);
                    self.push(Instruction::SwapAWithB, pos);
                    match op {
                        Operand::Less => self.push(Instruction::Less, pos),
                        Operand::LessOrEqual => self.push(Instruction::LessOrEqual, pos),
                        Operand::Equal => self.push(Instruction::Equal, pos),
                        Operand::GreaterOrEqual => self.push(Instruction::GreaterOrEqual, pos),
                        Operand::Greater => self.push(Instruction::Greater, pos),
                        _ => panic!("Unexpected CASE IS operator {:?}", op),
                    }
                    self.jump_if_false(next_case_label(case_blocks_len, has_else, idx), pos);
                    self.generate_block_instructions(case_block.statements);
                    self.jump("end-select", pos);
                }
                CaseExpression::Range(from, to) => {
                    // evaluate from -> A
                    self.generate_expression_instructions(from);
                    self.push(Instruction::LessOrEqual, pos);
                    self.jump_if_false(next_case_label(case_blocks_len, has_else, idx), pos);
                    // evaluate to -> A
                    self.generate_expression_instructions(to);
                    self.push(Instruction::GreaterOrEqual, pos);
                    self.jump_if_false(next_case_label(case_blocks_len, has_else, idx), pos);
                    self.generate_block_instructions(case_block.statements);
                    self.jump("end-select", pos);
                }
            }
            idx += 1;
        }
        match s.else_block {
            Some(e) => {
                self.label("case-else", pos);
                self.generate_block_instructions(e);
            }
            None => {}
        }
        self.label("end-select", pos);
    }
}
