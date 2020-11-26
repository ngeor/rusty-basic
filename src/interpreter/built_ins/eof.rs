// EOF(file-number%) -> checks if the end of file has been reached
use super::*;
use crate::interpreter::input::Input;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let file_handle: FileHandle = interpreter.context().get(0).unwrap().try_into()?;
    let file_input = interpreter
        .file_manager()
        .try_get_file_info_input_mut(&file_handle)?;
    let is_eof: bool = file_input.eof()?;
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Eof.into(), is_eof.into());
    Ok(())
}
