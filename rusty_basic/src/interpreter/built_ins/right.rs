use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use rusty_common::*;
use rusty_parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let count: usize = interpreter.context()[1].to_non_negative_int()?;
    let right_part: String = if s.len() > count {
        s.chars().skip(s.len() - count).collect()
    } else {
        s.to_owned()
    };
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Right, right_part);
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
        assert_prints!(r#"PRINT RIGHT$("hay", 0)"#, "");
        assert_prints!(r#"PRINT RIGHT$("hay", 1)"#, "y");
        assert_prints!(r#"PRINT RIGHT$("hay", 2)"#, "ay");
        assert_prints!(r#"PRINT RIGHT$("hay", 3)"#, "hay");
        assert_prints!(r#"PRINT RIGHT$("hay", 4)"#, "hay");
    }

    #[test]
    fn test_edge_cases() {
        assert_prints!(r#"PRINT RIGHT$("", 1)"#, "");
        assert_interpreter_err!(
            r#"PRINT RIGHT$("a", -1)"#,
            QError::IllegalFunctionCall,
            1,
            7
        );
    }
}
