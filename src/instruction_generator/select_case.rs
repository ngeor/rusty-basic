use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::parser::{CaseExpression, ExpressionNode, Operator, SelectCaseNode};

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
            let is_multi_expr = case_block.expression_list.len() > 1;
            self.generate_case_expressions(
                case_block.expression_list,
                next_case_label.as_str(),
                pos,
                idx,
            );
            // if we have a multi-expr CASE, we need a label marker for the CASE block statements to jump to
            if is_multi_expr {
                self.label(format!("case-statements{}", idx), pos);
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

    fn generate_case_expressions(
        &mut self,
        case_expressions: Vec<CaseExpression>,
        next_case_label: &str,
        pos: Location,
        idx: usize,
    ) {
        let expressions_len = case_expressions.len();
        if expressions_len > 1 {
            let opt_true_label = Some(format!("case-statements{}", idx));
            let mut i: usize = 0;
            for case_expr in case_expressions {
                let is_last = i + 1 == expressions_len;
                if i > 0 {
                    let inner_label = format!("case-multi-expr-{}-{}", idx, i);
                    self.label(inner_label, pos);
                }
                let n = if is_last {
                    next_case_label.to_owned()
                } else {
                    format!("case-multi-expr-{}-{}", idx, i + 1)
                };
                self.generate_case_expression(case_expr, &n, pos, opt_true_label.as_ref());
                i += 1;
            }
        } else {
            for case_expr in case_expressions {
                self.generate_case_expression(case_expr, next_case_label, pos, None);
            }
        }
    }

    fn generate_case_expression(
        &mut self,
        case_expression: CaseExpression,
        next_case_label: &str,
        pos: Location,
        opt_true_label: Option<&String>,
    ) {
        match case_expression {
            CaseExpression::Simple(e) => {
                self.generate_case_expr_simple(e, next_case_label, pos);
            }
            CaseExpression::Is(op, e) => {
                self.generate_case_expr_is(op, e, next_case_label, pos);
            }
            CaseExpression::Range(from, to) => {
                self.generate_case_expr_range(from, to, next_case_label, pos);
            }
        }
        if let Some(prefix) = opt_true_label {
            self.jump(prefix, pos);
        }
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
