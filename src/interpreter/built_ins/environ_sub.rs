// ENVIRON str-expr$ -> sets the variable.
// Parameter must be in the form of name=value or name value (TODO support the latter)
use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let s: &String = interpreter
        .context()
        .get(0)
        .unwrap()
        .try_into()
        .with_err_no_pos()?;
    let parts: Vec<&str> = s.split("=").collect();
    if parts.len() != 2 {
        Err(QError::from("Invalid expression. Must be name=value.")).with_err_no_pos()
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
    use crate::interpreter::stdlib::Stdlib;
    use crate::interpreter::test_utils::interpret;

    #[test]
    fn test_sub_call_environ() {
        let program = r#"
        ENVIRON "FOO=BAR"
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib().get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_sub_call_environ_by_ref() {
        let program = r#"
        A$ = "FOO1=BAR2"
        ENVIRON A$
        "#;
        let interpreter = interpret(program);
        assert_eq!(
            interpreter.stdlib().get_env_var(&"FOO1".to_string()),
            "BAR2"
        );
    }
}
