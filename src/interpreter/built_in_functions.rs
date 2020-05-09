use crate::common::*;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::{QualifiedName, TypeQualifier};
use crate::variant::Variant;

impl<S: Stdlib> Interpreter<S> {
    pub fn run_built_in_function(&mut self, function_name: &QualifiedName, _pos: Location) -> () {
        if function_name == &QualifiedName::new("ENVIRON", TypeQualifier::DollarString) {
            let v = self.context_mut().demand_sub().pop_front_unnamed();
            match v {
                Variant::VString(env_var_name) => {
                    let result = self.stdlib.get_env_var(&env_var_name);
                    self.function_result = Variant::VString(result);
                }
                _ => panic!("Type mismatch at ENVIRON$",),
            }
        } else {
            panic!("Unknown function {:?}", function_name);
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
