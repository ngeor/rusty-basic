// EOF(file-number%) -> checks if the end of file has been reached

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::{FileHandle, Location};
use crate::interpreter::{err, Interpreter, InterpreterError, Stdlib};
use crate::linter::{Error, ExpressionNode};
use crate::variant::Variant;

pub struct Eof {}

impl BuiltInLint for Eof {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        util::require_single_numeric_argument(args)
    }
}

impl BuiltInRun for Eof {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let v: Variant = interpreter.pop_unnamed_val().unwrap();
        let file_handle: FileHandle = match v {
            Variant::VFileHandle(f) => f,
            Variant::VInteger(i) => (i as u32).into(),
            _ => {
                return err("Invalid file handle in EOF", pos);
            }
        };
        let is_eof: bool = interpreter
            .file_manager
            .eof(file_handle)
            .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))?;
        interpreter.function_result = is_eof.into();
        Ok(())
    }
}
