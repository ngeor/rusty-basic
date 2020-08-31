// SYSTEM closes all open files and returns control to the operating system
// TODO close all open files

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};

pub struct System {}

impl BuiltInRun for System {
    fn run<S: Stdlib>(&self, _interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        panic!("Should have been handled at the IG level")
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn test_sub_call_system_no_args_allowed() {
        assert_linter_err!("SYSTEM 42", QError::ArgumentCountMismatch, 1, 1);
    }
}
