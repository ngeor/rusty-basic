use super::{Instruction, InstructionGenerator, Visitor};
use crate::common::*;
use crate::parser::{
    ConditionalBlockNode, DoLoopConditionKind, DoLoopConditionPosition, DoLoopNode, Expression,
    ExpressionNode, ForLoopNode, HasExpressionType, StatementNodes,
};

impl InstructionGenerator {
    pub fn generate_while_instructions(&mut self, w: ConditionalBlockNode, pos: Location) {
        self.label("while", pos);
        self.generate_expression_instructions(w.condition);
        self.jump_if_false("wend", pos);
        self.visit(w.statements);
        self.mark_statement_address(); // to be able to resume on error
        self.jump("while", pos);
        self.label("wend", pos);
    }

    fn store_counter(&mut self, counter_var: &Expression, pos: Location) {
        self.generate_store_instructions(counter_var.clone(), pos);
    }

    fn load_counter(&mut self, counter_var: &Expression, pos: Location) {
        self.generate_expression_instructions(counter_var.clone().at(pos));
    }

    pub fn generate_for_loop_instructions(&mut self, f: ForLoopNode, pos: Location) {
        let ForLoopNode {
            variable_name:
                Locatable {
                    element: counter_var_name,
                    ..
                },
            lower_bound,
            upper_bound,
            step,
            statements,
            ..
        } = f;
        // lower bound to A
        self.generate_expression_instructions_casting(
            lower_bound,
            counter_var_name.expression_type(),
        );
        // A to variable
        self.store_counter(&counter_var_name, pos);
        // upper bound to A
        self.generate_expression_instructions_casting(
            upper_bound,
            counter_var_name.expression_type(),
        );
        // A to C (upper bound to C)
        self.push(Instruction::CopyAToC, pos);
        // load the step expression
        match step {
            Some(s) => {
                let step_location = s.pos();
                // load 0 to B
                self.push_load(0, pos);
                self.push(Instruction::CopyAToB, pos);
                // load step to A
                self.generate_expression_instructions(s);
                // A to D (step is in D)
                self.push(Instruction::CopyAToD, pos);
                // is step < 0 ?
                self.push(Instruction::Less, pos);
                self.jump_if_false("test-positive-or-zero", pos);
                // negative step
                self.generate_for_loop_instructions_positive_or_negative_step(
                    &counter_var_name,
                    statements.clone(),
                    false,
                    pos,
                );
                // jump out
                self.jump("out-of-for", pos);
                // PositiveOrZero: ?
                self.label("test-positive-or-zero", pos);
                // need to load it again into A because the previous "LessThan" op overwrote A
                self.push(Instruction::CopyDToA, pos);
                // is step > 0 ?
                self.push(Instruction::Greater, pos);
                self.jump_if_false("zero", pos);
                // positive step
                self.generate_for_loop_instructions_positive_or_negative_step(
                    &counter_var_name,
                    statements,
                    true,
                    pos,
                );
                // jump out
                self.jump("out-of-for", pos);
                // Zero step
                self.label("zero", pos);
                self.push(Instruction::Throw(QError::ForLoopZeroStep), step_location);
                self.label("out-of-for", pos);
            }
            None => {
                self.push_load(1, pos);
                // A to D (step is in D)
                self.push(Instruction::CopyAToD, pos);
                self.generate_for_loop_instructions_positive_or_negative_step(
                    &counter_var_name,
                    statements,
                    true,
                    pos,
                );
                self.label("out-of-for", pos);
            }
        }
    }

    fn generate_for_loop_instructions_positive_or_negative_step(
        &mut self,
        counter_var_name: &Expression,
        statements: StatementNodes,
        is_positive: bool,
        pos: Location,
    ) {
        let loop_label = if is_positive {
            "positive-loop"
        } else {
            "negative-loop"
        };
        // loop point
        self.label(loop_label, pos);
        // upper bound from C to B
        self.push(Instruction::CopyCToB, pos);
        // counter to A
        self.load_counter(counter_var_name, pos);
        if is_positive {
            self.push(Instruction::LessOrEqual, pos);
        } else {
            self.push(Instruction::GreaterOrEqual, pos);
        }
        self.jump_if_false("out-of-for", pos);

        // push registers
        self.push(Instruction::PushRegisters, pos);

        // run loop body
        self.visit(statements);

        // to be able to resume after an error at the last statement and then pop registers
        self.mark_statement_address();
        self.push(Instruction::PopRegisters, pos);

        // increment step
        self.load_counter(counter_var_name, pos);
        // copy step from D to B
        self.push(Instruction::CopyDToB, pos);
        self.push(Instruction::Plus, pos);
        self.store_counter(counter_var_name, pos);

        // back to loop
        self.jump(loop_label, pos);
    }

    pub fn generate_do_loop_instructions(&mut self, do_loop_node: DoLoopNode, pos: Location) {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        match position {
            DoLoopConditionPosition::Top => {
                self.generate_do_loop_top(condition, statements, kind, pos)
            }
            DoLoopConditionPosition::Bottom => {
                self.generate_do_loop_bottom(condition, statements, kind, pos)
            }
        }
    }

    fn generate_do_loop_top(
        &mut self,
        condition: ExpressionNode,
        statements: StatementNodes,
        kind: DoLoopConditionKind,
        pos: Location,
    ) {
        self.label("do", pos);
        self.generate_expression_instructions(condition);
        if kind == DoLoopConditionKind::Until {
            self.push(Instruction::NotA, pos);
        }
        self.jump_if_false("loop", pos);
        self.visit(statements);
        self.mark_statement_address(); // to be able to resume on error
        self.jump("do", pos);
        self.label("loop", pos);
    }

    fn generate_do_loop_bottom(
        &mut self,
        condition: ExpressionNode,
        statements: StatementNodes,
        kind: DoLoopConditionKind,
        pos: Location,
    ) {
        self.label("do", pos);
        self.visit(statements);
        self.mark_statement_address(); // to be able to resume on error
        self.generate_expression_instructions(condition);
        if kind == DoLoopConditionKind::Until {
            self.push(Instruction::NotA, pos);
        }
        self.jump_if_false("loop", pos);
        self.jump("do", pos);
        self.label("loop", pos);
    }
}
