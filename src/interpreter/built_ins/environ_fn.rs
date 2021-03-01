// ENVIRON$ (env-variable$) -> returns the variable
// ENVIRON$ (n%) -> returns the nth variable (TODO support this)
use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let env_var_name: &String = (&interpreter.context()[0]).try_into().with_err_no_pos()?;
    let result = interpreter.stdlib().get_env_var(env_var_name);
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Environ.into(), result.into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_has_variable;
    use crate::assert_linter_err;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::stdlib::Stdlib;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_function_call_environ() {
        let program = r#"
        X$ = ENVIRON$("abc")
        Y$ = ENVIRON$("def")
        "#;
        let interpreter = interpret_with_env(program, |x| {
            x.stdlib_mut()
                .set_env_var("abc".to_string(), "foo".to_string())
        });
        assert_has_variable!(interpreter, "X$", "foo");
        assert_has_variable!(interpreter, "Y$", "");
    }

    #[test]
    fn test_function_call_environ_two_args_linter_err() {
        assert_linter_err!(
            r#"X$ = ENVIRON$("hi", "bye")"#,
            QError::ArgumentCountMismatch,
            1,
            6
        );
    }

    #[test]
    fn test_function_call_environ_numeric_arg_linter_err() {
        assert_linter_err!("X$ = ENVIRON$(42)", QError::ArgumentTypeMismatch, 1, 15);
    }
}
