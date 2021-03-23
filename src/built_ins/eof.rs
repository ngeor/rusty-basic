pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_numeric_argument()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Input;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
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
}

#[cfg(test)]
mod tests {}
