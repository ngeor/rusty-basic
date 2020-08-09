// EOF(file-number%) -> checks if the end of file has been reached

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{Error, ExpressionNode};
use crate::variant::Variant;

pub struct Eof {}

impl BuiltInLint for Eof {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        util::require_single_numeric_argument(args)
    }
}

impl BuiltInRun for Eof {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        let v: Variant = interpreter.pop_unnamed_val().unwrap();
        let file_handle: FileHandle = match v {
            Variant::VFileHandle(f) => f,
            Variant::VInteger(i) => (i as u32).into(),
            _ => {
                return Err("Invalid file handle in EOF".into()).with_err_no_pos();
            }
        };
        let is_eof: bool = interpreter
            .file_manager
            .eof(file_handle)
            .map_err(|e| e.to_string())
            .with_err_no_pos()?;
        interpreter.function_result = is_eof.into();
        Ok(())
    }
}
