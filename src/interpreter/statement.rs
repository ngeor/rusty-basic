use super::{Interpreter, Result, Stdlib};
use crate::parser::{BlockNode, StatementNode};

impl<S: Stdlib> Interpreter<S> {
    pub fn statement(&mut self, statement: &StatementNode) -> Result<()> {
        match statement {
            StatementNode::SubCall(name, args) => self.sub_call(name, args),
            StatementNode::ForLoop(f) => self.for_loop(f),
            StatementNode::IfBlock(i) => self.if_block(i),
            StatementNode::Assignment(left_side, right_side) => {
                self.assignment(left_side, right_side).map(|_| ())
            }
            StatementNode::Whitespace(_) => Ok(()),
        }
    }

    pub fn statements(&mut self, statements: &BlockNode) -> Result<()> {
        for statement in statements {
            match self.statement(statement) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(())
    }
}
