use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;
use rusty_parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let result = s.trim_start().to_owned();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::LTrim, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints_exact;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_happy_flow() {
        let program = r#"PRINT LTRIM$("  * hello world *  ")"#;
        assert_prints_exact!(program, "* hello world *  ", "");
    }
}
