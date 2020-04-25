use super::{Interpreter, Result, Stdlib};
use crate::parser::{BlockNode, StatementNode};

pub trait StatementRunner<T> {
    fn run(&mut self, statement: &T) -> Result<()>;
}

impl<S: Stdlib> StatementRunner<StatementNode> for Interpreter<S> {
    fn run(&mut self, statement: &StatementNode) -> Result<()> {
        match statement {
            StatementNode::SubCall(name, args) => self.sub_call(name, args),
            StatementNode::ForLoop(f) => self.for_loop(f),
            StatementNode::IfBlock(i) => self.if_block(i),
            StatementNode::Assignment(left_side, right_side) => {
                self.assignment(left_side, right_side)
            }
            StatementNode::While(w) => self.while_wend(w),
            StatementNode::Const(left, right, _) => self.handle_const(left, right),
        }
    }
}

impl<T: StatementRunner<StatementNode>> StatementRunner<BlockNode> for T {
    fn run(&mut self, statements: &BlockNode) -> Result<()> {
        for statement in statements {
            match self.run(statement) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(())
    }
}

impl<T: StatementRunner<BlockNode>> StatementRunner<Option<BlockNode>> for T {
    fn run(&mut self, statements: &Option<BlockNode>) -> Result<()> {
        match statements {
            Some(x) => self.run(x),
            None => Ok(()),
        }
    }
}
