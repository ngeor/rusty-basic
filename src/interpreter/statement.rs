use super::*;
use crate::common::Result;
use crate::parser::*;

impl<S: Stdlib> Interpreter<S> {
    pub fn statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::SubCall(name, args) => self.sub_call(name, args),
            Statement::ForLoop(f) => self.for_loop(f),
            Statement::IfBlock(i) => self.if_block(i),
            Statement::Assignment(left_side, right_side) => self.assignment(left_side, right_side),
            Statement::Whitespace(_) => Ok(()),
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
}
