use rusty_parser::built_ins::built_in_function::BuiltInFunction;

use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let start: usize = interpreter.context()[1].to_positive_int()?;
    let length: Option<usize> = match interpreter.context().variables().get(2) {
        Some(v) => Some(v.to_non_negative_int()?),
        None => None,
    };
    let result: String = do_mid(s, start, length)?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Mid, result);
    Ok(())
}

fn do_mid(s: &str, start: usize, opt_length: Option<usize>) -> Result<String, RuntimeError> {
    let start_index: usize = start - 1;
    match opt_length {
        Some(length) => {
            let end: usize = if start_index + length > s.len() {
                s.len()
            } else {
                start_index + length
            };
            Ok(s.get(start_index..end).unwrap_or_default().to_string())
        }
        None => Ok(s.get(start_index..).unwrap_or_default().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_mid_happy_flow() {
        assert_prints!(r#"PRINT MID$("hay", 1)"#, "hay");
        assert_prints!(r#"PRINT MID$("hay", 2)"#, "ay");
        assert_prints!(r#"PRINT MID$("hay", 1, 1)"#, "h");
        assert_prints!(r#"PRINT MID$("hay", 2, 2)"#, "ay");
        assert_prints!(r#"PRINT MID$("hay", 2, 20)"#, "ay");
    }

    #[test]
    fn test_mid_edge_cases() {
        assert_prints!(r#"PRINT MID$("", 1)"#, "");
        assert_prints!(r#"PRINT MID$("hay", 4)"#, "");
        assert_prints!(r#"PRINT MID$("hay", 1, 0)"#, "");
        assert_interpreter_err!(
            r#"PRINT MID$("hay", 0)"#,
            RuntimeError::IllegalFunctionCall,
            1,
            7
        );
        assert_interpreter_err!(
            r#"PRINT MID$("hay", 1, -1)"#,
            RuntimeError::IllegalFunctionCall,
            1,
            7
        );
    }
}
