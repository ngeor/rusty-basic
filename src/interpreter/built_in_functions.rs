use crate::common::*;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::BuiltInFunction;
use crate::variant::Variant;
use std::convert::TryInto;

impl<S: Stdlib> Interpreter<S> {
    pub fn run_built_in_function(&mut self, function_name: &BuiltInFunction, _pos: Location) {
        match function_name {
            BuiltInFunction::Environ => self.run_environ(),
            BuiltInFunction::Len => self.run_len(),
        }
    }

    fn run_environ(&mut self) {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        match v {
            Variant::VString(env_var_name) => {
                let result = self.stdlib.get_env_var(&env_var_name);
                self.function_result = Variant::VString(result);
            }
            _ => panic!("Type mismatch at ENVIRON$",),
        }
    }

    fn run_len(&mut self) {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        self.function_result = match v {
            Variant::VSingle(_) => Variant::VInteger(4),
            Variant::VDouble(_) => Variant::VInteger(8),
            Variant::VString(v) => Variant::VInteger(v.len().try_into().unwrap()),
            Variant::VInteger(_) => Variant::VInteger(2),
            Variant::VLong(_) => Variant::VInteger(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_has_variable;
    use crate::assert_linter_err;
    use crate::interpreter::Stdlib;
    use crate::linter::LinterError;
    use crate::variant::Variant;

    mod environ {
        use super::*;
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
        fn test_function_call_environ_no_args_linter_err() {
            assert_linter_err!("X$ = ENVIRON$()", LinterError::ArgumentCountMismatch, 1, 6);
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

    mod len {
        use super::*;

        #[test]
        fn test_len_string() {
            let program = r#"PRINT LEN("hello")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["5"]);
        }

        #[test]
        fn test_len_float_variable() {
            let program = "
            A = 3.14
            PRINT LEN(A)
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["4"]);
        }

        #[test]
        fn test_len_double_variable() {
            let program = "
            A# = 3.14
            PRINT LEN(A#)
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["8"]);
        }

        #[test]
        fn test_len_integer_variable() {
            let program = "
            A% = 42
            PRINT LEN(A%)
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["2"]);
        }

        #[test]
        fn test_len_long_variable() {
            let program = "
            A& = 42
            PRINT LEN(A&)
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["4"]);
        }

        #[test]
        fn test_len_integer_expression_error() {
            let program = "PRINT LEN(42)";
            assert_linter_err!(program, LinterError::VariableRequired, 1, 11);
        }

        #[test]
        fn test_len_integer_const_error() {
            let program = "
            CONST X = 42
            PRINT LEN(X)
            ";
            assert_linter_err!(program, LinterError::VariableRequired, 3, 23);
        }

        #[test]
        fn test_len_no_arguments_error() {
            let program = "PRINT LEN()";
            assert_linter_err!(program, LinterError::ArgumentCountMismatch, 1, 7);
        }

        #[test]
        fn test_len_two_arguments_error() {
            let program = r#"PRINT LEN("a", "b")"#;
            assert_linter_err!(program, LinterError::ArgumentCountMismatch, 1, 7);
        }

        #[test]
        fn test_len_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_linter_err!(program, LinterError::SyntaxError, 1, 7);
        }
    }
}
