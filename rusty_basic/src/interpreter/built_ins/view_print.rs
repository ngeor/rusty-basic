use crate::RuntimeError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    if interpreter.context().variables().len() > 0 {
        let start_row = interpreter.context()[0].to_positive_int()?;
        let end_row = interpreter.context()[1].to_positive_int()?;
        if start_row >= end_row {
            Err(RuntimeError::IllegalFunctionCall)
        } else {
            // we have args
            interpreter.screen_mut().set_view_print(start_row, end_row);
            Ok(())
        }
    } else {
        // reset full view
        interpreter.screen_mut().reset_view_print();
        Ok(())
    }
}
