use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;
use rusty_parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let result = s.to_ascii_lowercase();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::LCase, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_happy_flow() {
        assert_prints!(r#"PRINT LCASE$("hay")"#, "hay");
        assert_prints!(r#"PRINT LCASE$("WORLD")"#, "world");
        assert_prints!(r#"PRINT LCASE$("Oops")"#, "oops");
        assert_prints!(r#"PRINT LCASE$("")"#, "");
    }
}
