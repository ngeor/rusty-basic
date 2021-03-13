// EOF(file-number%) -> checks if the end of file has been reached
use super::*;
use crate::common::{FileHandle, QError};
use crate::interpreter::input::Input;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let file_handle: FileHandle = FileHandle::try_from(&interpreter.context()[0])?;
    let file_input = interpreter
        .file_manager()
        .try_get_file_info_input(&file_handle)?;
    let is_eof: bool = file_input.eof()?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Eof, is_eof);
    Ok(())
}
