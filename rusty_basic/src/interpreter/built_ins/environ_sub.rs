use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::interpreter::Stdlib;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let s: &str = interpreter.context()[0].to_str_unchecked();
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        Err(RuntimeError::Other(
            "Invalid expression. Must be name=value.".to_string(),
        ))
    } else {
        let name = parts[0].to_string();
        let value = parts[1].to_string();
        interpreter.stdlib_mut().set_env_var(name, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::interpret;
    use crate::interpreter::Stdlib;

    #[test]
    fn test_sub_call_environ() {
        let program = r#"
        ENVIRON "FOO=BAR"
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib().get_env_var("FOO"), "BAR");
    }

    #[test]
    fn test_sub_call_environ_by_ref() {
        let program = r#"
        A$ = "FOO1=BAR2"
        ENVIRON A$
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib().get_env_var("FOO1"), "BAR2");
    }
}
