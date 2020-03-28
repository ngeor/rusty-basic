use super::context::ReadWriteContext;
use super::Interpreter;
use super::Stdlib;
use crate::common::Result;
use crate::parser::*;
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::SubCall(name, args) => self.sub_call(name, args),
            Statement::ForLoop(i, a, b, statements) => self.for_loop(i, a, b, statements),
            Statement::IfBlock(i) => self._if_block(i),
            Statement::Assignment(left_side, right_side) => self._assignment(left_side, right_side),
        }
    }

    pub fn statements(&mut self, statements: &Block) -> Result<()> {
        for statement in statements {
            match self.statement(statement) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(())
    }

    fn _if_block(&mut self, if_block: &IfBlock) -> Result<()> {
        let if_condition_expr = &if_block.if_block.condition;
        let if_condition_var = self.evaluate_expression(if_condition_expr)?;
        if if_condition_var.is_true() {
            self.statements(&if_block.if_block.block)
        } else {
            for else_if_block in &if_block.else_if_blocks {
                let if_condition_expr = &else_if_block.condition;
                let if_condition_var = self.evaluate_expression(if_condition_expr)?;
                if if_condition_var.is_true() {
                    return self.statements(&else_if_block.block)
                }
            }

            match &if_block.else_block {
                Some(e) => self.statements(&e),
                None => Ok(())
            }
        }
    }

    fn _assignment(
        &mut self,
        left_side: &QName,
        right_side: &Expression,
    ) -> Result<()> {
        let val = self.evaluate_expression(right_side)?;
        self.set_variable(left_side, val)
    }
}
