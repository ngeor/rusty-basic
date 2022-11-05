use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use rusty_common::*;
use rusty_parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let count: usize = interpreter.context()[1].to_non_negative_int()?;
    let left_part: String = s.chars().take(count).collect();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Left, left_part);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use rusty_common::*;

    #[test]
    fn test_happy_flow() {
        assert_prints!(r#"PRINT LEFT$("hay", 0)"#, "");
        assert_prints!(r#"PRINT LEFT$("hay", 1)"#, "h");
        assert_prints!(r#"PRINT LEFT$("hay", 2)"#, "ha");
        assert_prints!(r#"PRINT LEFT$("hay", 3)"#, "hay");
        assert_prints!(r#"PRINT LEFT$("hay", 4)"#, "hay");
    }

    #[test]
    fn test_edge_cases() {
        assert_prints!(r#"PRINT LEFT$("", 1)"#, "");
        assert_interpreter_err!(r#"PRINT LEFT$("a", -1)"#, QError::IllegalFunctionCall, 1, 7);
    }
}
