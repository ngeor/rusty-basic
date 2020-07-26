// SYSTEM closes all open files and returns control to the operating system
// TODO close all open files

use super::{BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::linter::{err_no_pos, Error, ExpressionNode, LinterError};

pub struct System {}

impl BuiltInLint for System {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 0 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for System {
    fn run<S: Stdlib>(
        &self,
        _interpreter: &mut Interpreter<S>,
        _pos: Location,
    ) -> Result<(), InterpreterError> {
        panic!("Should have been handled at the IG level")
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    #[test]
    fn test_sub_call_system_no_args_allowed() {
        assert_linter_err!("SYSTEM 42", LinterError::ArgumentCountMismatch, 1, 1);
    }
}
