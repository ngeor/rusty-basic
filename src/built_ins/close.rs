// CLOSE
// TODO : support integer as argument e.g. CLOSE 1 instead of just CLOSE #1
// TODO : close without arguments closes all files

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};

pub struct Close {}

impl BuiltInRun for Close {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_handle = interpreter.pop_file_handle();
        interpreter.file_manager.close(file_handle);
        Ok(())
    }
}
