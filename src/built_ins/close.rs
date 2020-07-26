// CLOSE
// TODO : support integer as argument e.g. CLOSE 1 instead of just CLOSE #1
// TODO : close without arguments closes all files

use super::{BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::linter::{err_l, err_no_pos, Error, Expression, ExpressionNode, LinterError};

pub struct Close {}

impl BuiltInLint for Close {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            match args[0].as_ref() {
                Expression::FileHandle(_) => Ok(()),
                _ => err_l(LinterError::ArgumentTypeMismatch, &args[0]),
            }
        }
    }
}

impl BuiltInRun for Close {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        _pos: Location,
    ) -> Result<(), InterpreterError> {
        let file_handle = interpreter.pop_file_handle();
        interpreter.file_manager.close(file_handle);
        Ok(())
    }
}
