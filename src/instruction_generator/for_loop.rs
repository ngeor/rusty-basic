use super::{Instruction, InstructionGenerator, Result};
use crate::common::*;
use crate::linter::{ForLoopNode, QualifiedName, StatementNodes};
use crate::variant::Variant;

impl InstructionGenerator {
    pub fn generate_for_loop_instructions(&mut self, f: ForLoopNode, pos: Location) -> Result<()> {
        let ForLoopNode {
            variable_name,
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter: _,
        } = f;
        let counter_var_name = variable_name.strip_location();
        // lower bound to A
        self.generate_expression_instructions(lower_bound)?;
        // A to variable
        self.push(Instruction::Store(counter_var_name.clone()), pos);
        // upper bound to A
        self.generate_expression_instructions(upper_bound)?;
        // A to C (upper bound to C)
        self.push(Instruction::CopyAToC, pos);
        // load the step expression
        match step {
            Some(s) => {
                let step_location = s.location();
                // load 0 to B
                self.push(Instruction::Load(Variant::VInteger(0)), pos);
                self.push(Instruction::CopyAToB, pos);
                // load step to A
                self.generate_expression_instructions(s)?;
                // A to D (step is in D)
                self.push(Instruction::CopyAToD, pos);
                // is step < 0 ?
                self.push(Instruction::LessThan, pos);
                self.jump_if_false("test-positive-or-zero", pos);
                // negative step
                self.generate_for_loop_instructions_positive_or_negative_step(
                    counter_var_name.clone(),
                    statements.clone(),
                    false,
                    pos,
                )?;
                // jump out
                self.jump("out-of-for", pos);
                // PositiveOrZero: ?
                self.label("test-positive-or-zero", pos);
                // need to load it again into A because the previous "LessThan" op overwrote A
                self.push(Instruction::CopyDToA, pos);
                // is step > 0 ?
                self.push(Instruction::GreaterThan, pos);
                self.jump_if_false("zero", pos);
                // positive step
                self.generate_for_loop_instructions_positive_or_negative_step(
                    counter_var_name,
                    statements,
                    true,
                    pos,
                )?;
                // jump out
                self.jump("out-of-for", pos);
                // Zero step
                self.label("zero", pos);
                self.push(
                    Instruction::Throw(format!("Step cannot be zero")),
                    step_location,
                );
                self.label("out-of-for", pos);
                Ok(())
            }
            None => {
                self.push(Instruction::Load(Variant::VInteger(1)), pos);
                // A to D (step is in D)
                self.push(Instruction::CopyAToD, pos);
                self.generate_for_loop_instructions_positive_or_negative_step(
                    counter_var_name,
                    statements,
                    true,
                    pos,
                )?;
                self.label("out-of-for", pos);
                Ok(())
            }
        }
    }

    fn generate_for_loop_instructions_positive_or_negative_step(
        &mut self,
        counter_var_name: QualifiedName,
        statements: StatementNodes,
        is_positive: bool,
        pos: Location,
    ) -> Result<()> {
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
        self.push(Instruction::CopyVarToA(counter_var_name.clone()), pos);
        if is_positive {
            self.push(Instruction::LessOrEqualThan, pos);
        } else {
            self.push(Instruction::GreaterOrEqualThan, pos);
        }
        self.jump_if_false("out-of-for", pos);

        self.push(Instruction::PushRegisters, pos);
        // run loop body
        self.generate_block_instructions(statements)?;
        self.push(Instruction::PopRegisters, pos);

        // increment step
        self.push(Instruction::CopyVarToA(counter_var_name.clone()), pos);
        // copy step from D to B
        self.push(Instruction::CopyDToB, pos);
        self.push(Instruction::Plus, pos);
        self.push(Instruction::Store(counter_var_name), pos);

        // back to loop
        self.jump(loop_label, pos);
        Ok(())
    }
}
