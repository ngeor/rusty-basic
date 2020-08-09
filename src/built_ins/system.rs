// SYSTEM closes all open files and returns control to the operating system
// TODO close all open files

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterError, LinterErrorNode};

pub struct System {}

impl BuiltInLint for System {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        if args.len() != 0 {
            Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for System {
    fn run<S: Stdlib>(
        &self,
        _interpreter: &mut Interpreter<S>,
    ) -> Result<(), InterpreterErrorNode> {
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
