use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::Stdlib;
use crate::parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let env_var_name: &str = interpreter.context()[0].to_str_unchecked();
    let result = interpreter.stdlib().get_env_var(env_var_name);
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Environ, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_has_variable;
    use crate::interpreter::test_utils::*;
    use crate::interpreter::Stdlib;

    #[test]
    fn test_function_call_environ() {
        let program = r#"
        X$ = ENVIRON$("abc")
        Y$ = ENVIRON$("def")
        "#;
        let mut stdlib = MockStdlib::default();
        stdlib.set_env_var("abc".to_string(), "foo".to_string());
        let interpreter = interpret_with_env(program, stdlib);
        assert_has_variable!(interpreter, "X$", "foo");
        assert_has_variable!(interpreter, "Y$", "");
    }
}
