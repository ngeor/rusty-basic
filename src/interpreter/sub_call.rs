use super::variable_setter::VariableSetter;
use super::{Interpreter, InterpreterError, Result, Stdlib, Variant};
use crate::common::HasLocation;
use crate::interpreter::built_in_subs;
use crate::interpreter::user_defined_sub;
use crate::parser::{BareNameNode, ExpressionNode, NameNode, ResolveIntoRef, TypeQualifier};

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn sub_call(&mut self, name_node: &BareNameNode, args: &Vec<ExpressionNode>) -> Result<()> {
        let bare_name = name_node.as_ref();
        if bare_name == "PRINT" {
            self._do_print(args)
        } else if bare_name == "INPUT" {
            self._do_input(args)
        } else if bare_name == "SYSTEM" {
            self.stdlib.system();
            Ok(())
        } else if built_in_subs::supports_sub(name_node) {
            let arg_values: Vec<Variant> = self.evaluate_arguments(args)?;
            built_in_subs::call_sub(&mut self.stdlib, name_node, args, arg_values)
        } else {
            if user_defined_sub::supports_sub(self, name_node) {
                let arg_values: Vec<Variant> = self.evaluate_arguments(args)?;
                user_defined_sub::call_sub(self, name_node, args, arg_values)
            } else {
                Err(InterpreterError::new_with_pos(
                    format!("Unknown sub {}", bare_name),
                    name_node.location(),
                ))
            }
        }
    }

    fn _do_print(&mut self, args: &Vec<ExpressionNode>) -> Result<()> {
        let mut strings: Vec<String> = vec![];
        for a in args {
            strings.push(self._do_print_map_arg(a)?);
        }
        self.stdlib.print(strings);
        Ok(())
    }

    fn _do_print_map_arg(&mut self, arg: &ExpressionNode) -> Result<String> {
        let evaluated = self.evaluate_expression(arg)?;
        Ok(evaluated.to_string())
    }

    fn _do_input(&mut self, args: &Vec<ExpressionNode>) -> Result<()> {
        for a in args {
            self._do_input_one(a)?;
        }
        Ok(())
    }

    fn _do_input_one(&mut self, expression: &ExpressionNode) -> Result<()> {
        match expression {
            ExpressionNode::VariableName(n) => self._do_input_one_var(n),
            _ => Err(InterpreterError::new_with_pos(
                format!("Expected variable name, was {:?}", expression),
                expression.location(),
            )),
        }
    }

    fn _do_input_one_var(&mut self, var_name: &NameNode) -> Result<()> {
        let raw_input: String = self
            .stdlib
            .input()
            .map_err(|e| InterpreterError::new_with_pos(e.to_string(), var_name.location()))?;
        let q: TypeQualifier = var_name.resolve_into(self);
        let variable_value = match q {
            TypeQualifier::BangSingle => Variant::from(
                parse_single_input(raw_input)
                    .map_err(|e| InterpreterError::new_with_pos(e, var_name.location()))?,
            ),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(
                parse_int_input(raw_input)
                    .map_err(|e| InterpreterError::new_with_pos(e, var_name.location()))?,
            ),
            _ => unimplemented!(),
        };
        self.set_variable(var_name, variable_value).map(|_| ())
    }
}

fn parse_single_input(s: String) -> std::result::Result<f32, String> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| format!("Could not parse {} as float: {}", s, e))
    }
}

fn parse_int_input(s: String) -> std::result::Result<i32, String> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| format!("Could not parse {} as int: {}", s, e))
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::test_utils::*;
    use crate::interpreter::Stdlib;

    mod input {
        mod unqualified_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_empty() {
                assert_input("", "N", 0.0_f32);
            }

            #[test]
            fn test_input_zero() {
                assert_input("0", "N", 0.0_f32);
            }

            #[test]
            fn test_input_single() {
                assert_input("1.1", "N", 1.1_f32);
            }

            #[test]
            fn test_input_negative() {
                assert_input("-1.2345", "N", -1.2345_f32);
            }

            #[test]
            fn test_input_explicit_positive() {
                assert_input("+3.14", "N", 3.14_f32);
            }
        }

        mod string_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_hello() {
                assert_input("hello", "A$", "hello");
            }

            #[test]
            fn test_input_does_not_trim_new_line() {
                assert_input("hello\r\n", "A$", "hello\r\n");
            }
        }

        mod int_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_42() {
                assert_input("42", "A%", 42);
            }
        }
    }

    #[test]
    fn test_sub_call_environ() {
        let program = r#"
        ENVIRON "FOO=BAR"
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_interpret_sub_call_user_defined_no_args() {
        let program = r#"
        DECLARE SUB Hello
        Hello
        SUB Hello
            ENVIRON "FOO=BAR"
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_interpret_sub_call_user_defined_two_args() {
        let program = r#"
        DECLARE SUB Hello(N$, V$)
        Hello "FOO", "BAR"
        SUB Hello(N$, V$)
            ENVIRON N$ + "=" + V$
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }
}
