// ENVIRON$ (env-variable$) -> returns the variable
// ENVIRON$ (n%) -> returns the nth variable (TODO support this)

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::linter::{Error, ExpressionNode};
use crate::variant::Variant;

pub struct Environ {}

impl BuiltInLint for Environ {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        util::require_single_string_argument(args)
    }
}

impl BuiltInRun for Environ {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        _pos: Location,
    ) -> Result<(), InterpreterError> {
        let v = interpreter.pop_unnamed_val().unwrap();
        match v {
            Variant::VString(env_var_name) => {
                let result = interpreter.stdlib.get_env_var(&env_var_name);
                interpreter.function_result = Variant::VString(result);
                Ok(())
            }
            _ => panic!("Type mismatch at ENVIRON$",),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_has_variable;
    use crate::assert_linter_err;
    use crate::interpreter::test_utils::*;
    use crate::interpreter::Stdlib;
    use crate::linter::LinterError;

    #[test]
    fn test_function_call_environ() {
        let program = r#"
            X$ = ENVIRON$("abc")
            Y$ = ENVIRON$("def")
            "#;
        let mut stdlib = MockStdlib::new();
        stdlib.set_env_var("abc".to_string(), "foo".to_string());
        let interpreter = interpret_with_stdlib(program, stdlib);
        assert_has_variable!(interpreter, "X$", "foo");
        assert_has_variable!(interpreter, "Y$", "");
    }

    #[test]
    fn test_function_call_environ_two_args_linter_err() {
        assert_linter_err!(
            r#"X$ = ENVIRON$("hi", "bye")"#,
            LinterError::ArgumentCountMismatch,
            1,
            6
        );
    }

    #[test]
    fn test_function_call_environ_numeric_arg_linter_err() {
        assert_linter_err!(
            "X$ = ENVIRON$(42)",
            LinterError::ArgumentTypeMismatch,
            1,
            15
        );
    }
}
