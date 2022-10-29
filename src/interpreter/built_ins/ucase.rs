use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let result = s.to_ascii_uppercase();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::UCase, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_happy_flow() {
        assert_prints!(r#"PRINT UCASE$("hay")"#, "HAY");
        assert_prints!(r#"PRINT UCASE$("WORLD")"#, "WORLD");
        assert_prints!(r#"PRINT UCASE$("Oops")"#, "OOPS");
        assert_prints!(r#"PRINT UCASE$("")"#, "");
    }
}
