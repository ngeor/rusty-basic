// EOF(file-number%) -> checks if the end of file has been reached

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::variant::Variant;

pub struct Eof {}

impl BuiltInRun for Eof {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v: Variant = interpreter.pop_unnamed_val().unwrap();
        let file_handle: FileHandle = match v {
            Variant::VFileHandle(f) => f,
            Variant::VInteger(i) => (i as u32).into(),
            _ => {
                panic!("Invalid file handle in EOF, linter should have caught this");
            }
        };
        let is_eof: bool = interpreter
            .file_manager
            .eof(file_handle)
            .map_err(|e| e.into())
            .with_err_no_pos()?;
        interpreter.function_result = is_eof.into();
        Ok(())
    }
}
