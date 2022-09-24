use super::{Instruction, InstructionGenerator, Visitor};
use crate::common::*;
use crate::parser::{
    CaseBlockNode, CaseExpression, ExpressionNode, Operator, SelectCaseNode, StatementNodes,
};

impl InstructionGenerator {
    pub fn generate_select_case_instructions(&mut self, s: SelectCaseNode, pos: Location) {
        let SelectCaseNode {
            expr,
            case_blocks,
            else_block,
            ..
        } = s;
        self.generate_eval_select_case_expr(expr, pos);
        self.generate_case_blocks(case_blocks, else_block.is_some(), pos);
        self.generate_else_block(else_block, pos);
        // need to pop value from stack because it was pushed by `generate_eval_select_case_expr`
        self.push(Instruction::PopValueStackIntoA, pos);
        self.label(labels::end_select(), pos);
    }

    /// Evaluate SELECT CASE x into A
    fn generate_eval_select_case_expr(&mut self, expr: ExpressionNode, pos: Location) {
        self.generate_expression_instructions(expr);
        self.push(Instruction::PushAToValueStack, pos);
    }

    fn generate_case_blocks(
        &mut self,
        case_blocks: Vec<CaseBlockNode>,
        has_else: bool,
        pos: Location,
    ) {
        let mut case_block_index: usize = 0;
        let case_blocks_len = case_blocks.len();
        for case_block in case_blocks {
            // mark the beginning of this case block
            self.label(labels::case_block(case_block_index), pos);
            // where to jump out from here if the case block isn't matching
            let next_case_label =
                labels::next_case_label(case_blocks_len, has_else, case_block_index);
            let is_multi_expr = case_block.expression_list.len() > 1;
            self.generate_case_expressions(
                case_block.expression_list,
                next_case_label.as_str(),
                pos,
                case_block_index,
            );
            // if we have a multi-expr CASE, we need a label marker for the CASE block statements to jump to
            if is_multi_expr {
                // mark beginning of CASE block statements
                self.label(labels::case_statements(case_block_index), pos);
            }
            // run matched CASE block statements
            self.visit(case_block.statements);
            // jump out of SELECT
            self.jump(labels::end_select(), pos);
            case_block_index += 1;
        }
    }

    fn generate_else_block(&mut self, else_block: Option<StatementNodes>, pos: Location) {
        if let Some(e) = else_block {
            self.label(labels::case_else(), pos);
            self.visit(e);
        }
    }

    fn generate_case_expressions(
        &mut self,
        case_expressions: Vec<CaseExpression>,
        next_case_label: &str,
        pos: Location,
        case_block_index: usize,
    ) {
        let expressions_len = case_expressions.len();
        if expressions_len > 1 {
            // multi expr
            let mut case_expr_index: usize = 0;
            for case_expr in case_expressions {
                let is_last = case_expr_index + 1 == expressions_len;
                if case_expr_index > 0 {
                    // mark the beginning of the evaluation of this CASE expr,
                    // which will be where we jump to if the previous is false
                    let inner_label = labels::case_expr(case_block_index, case_expr_index);
                    self.label(inner_label, pos);
                }
                let next_label = if is_last {
                    // if this is the last expr, we jump to the next case label
                    next_case_label.to_owned()
                } else {
                    // otherwise we jump to the next CASE expr within the same CASE block
                    labels::case_expr(case_block_index, case_expr_index + 1)
                };
                self.generate_case_expression(case_expr, &next_label, pos);
                if !is_last {
                    // if this expression matched, jump directly into the CASE block statements and do not evaluate the rest
                    self.jump(labels::case_statements(case_block_index), pos);
                }
                case_expr_index += 1;
            }
        } else {
            // single expr is simpler
            for case_expr in case_expressions {
                self.generate_case_expression(case_expr, next_case_label, pos);
            }
        }
    }

    fn generate_case_expression(
        &mut self,
        case_expression: CaseExpression,
        next_case_label: &str,
        pos: Location,
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
    }

    fn generate_case_expr_simple(
        &mut self,
        e: ExpressionNode,
        next_case_label: &str,
        pos: Location,
    ) {
        self.generate_comparison_expr(e, pos);
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
        self.generate_comparison_expr(e, pos);
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
        self.generate_comparison_expr(from, pos);
        // compare select expr with lower bound, must be >=
        self.push(Instruction::GreaterOrEqual, pos);
        // jump out if it isn't >=
        self.jump_if_false(next_case_label, pos);
        // evaluate to -> A
        self.generate_comparison_expr(to, pos);
        self.push(Instruction::LessOrEqual, pos);
        self.jump_if_false(next_case_label, pos);
    }

    fn generate_comparison_expr(&mut self, comparison_expr: ExpressionNode, pos: Location) {
        // evaluate the comparison expression into A
        self.generate_expression_instructions(comparison_expr);
        // copy from -> B
        self.push(Instruction::CopyAToB, pos);
        // get select expr back into A
        self.push(Instruction::PopValueStackIntoA, pos);
        self.push(Instruction::PushAToValueStack, pos);
    }
}

mod labels {
    pub fn case_block(case_block_index: usize) -> String {
        format!("case{}", case_block_index)
    }

    pub fn case_expr(case_block_index: usize, case_expr_index: usize) -> String {
        format!("case-multi-expr-{}-{}", case_block_index, case_expr_index)
    }

    pub fn case_statements(case_block_index: usize) -> String {
        format!("case-statements{}", case_block_index)
    }

    pub fn case_else() -> String {
        "case-else".to_string()
    }

    pub fn end_select() -> String {
        "end-select".to_string()
    }

    pub fn next_case_label(
        case_blocks_len: usize,
        has_else_block: bool,
        case_block_index: usize,
    ) -> String {
        if case_block_index + 1 < case_blocks_len {
            case_block(case_block_index + 1)
        } else if has_else_block {
            case_else()
        } else {
            end_select()
        }
    }
}
