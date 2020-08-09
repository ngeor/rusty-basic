// CLOSE
// TODO : support integer as argument e.g. CLOSE 1 instead of just CLOSE #1
// TODO : close without arguments closes all files

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{Error, Expression, ExpressionNode, LinterError};

pub struct Close {}

impl BuiltInLint for Close {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            match args[0].as_ref() {
                Expression::FileHandle(_) => Ok(()),
                _ => Err(LinterError::ArgumentTypeMismatch).with_err_at(&args[0]),
            }
        }
    }
}

impl BuiltInRun for Close {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        let file_handle = interpreter.pop_file_handle();
        interpreter.file_manager.close(file_handle);
        Ok(())
    }
}
