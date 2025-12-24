use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Input;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;
use rusty_parser::{specific::FileHandle, BuiltInFunction};

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let file_handle: FileHandle = interpreter.context()[0].to_file_handle()?;
    let file_input = interpreter
        .file_manager()
        .try_get_file_info_input(&file_handle)?;
    let is_eof: bool = file_input.eof()?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Eof, is_eof);
    Ok(())
}
