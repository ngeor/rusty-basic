use super::variable_setter::VariableSetter;
use super::{Interpreter, InterpreterError, Result, Stdlib, Variant};
use crate::common::HasLocation;
use crate::parser::{BareNameNode, ExpressionNode, NameNode, ResolvesQualifier, TypeQualifier};

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn sub_call(&mut self, name: &BareNameNode, args: &Vec<ExpressionNode>) -> Result<()> {
        let raw_name = name.element();
        if raw_name == "PRINT" {
            self._do_print(args)
        } else if raw_name == "INPUT" {
            self._do_input(args)
        } else if raw_name == "SYSTEM" {
            self.stdlib.system();
            Ok(())
        } else if raw_name == "ENVIRON" {
            self._do_environ_sub(name, args)
        } else {
            Err(InterpreterError::new_with_pos(
                format!("Unknown sub {}", raw_name),
                name.location(),
            ))
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
        let variable_value = match var_name.qualifier(self) {
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

    fn _do_environ_sub(
        &mut self,
        sub_name_node: &BareNameNode,
        args: &Vec<ExpressionNode>,
    ) -> Result<()> {
        if args.len() != 1 {
            return Err(InterpreterError::new_with_pos(
                "ENVIRON requires exactly 1 argument",
                sub_name_node.location(),
            ));
        }

        let arg_value = self.evaluate_expression(&args[0])?;
        match arg_value {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    Err(InterpreterError::new_with_pos(
                        "Invalid expression. Must be name=value.",
                        args[0].location(),
                    ))
                } else {
                    self.stdlib
                        .set_env_var(parts[0].to_string(), parts[1].to_string());
                    Ok(())
                }
            }
            _ => Err(InterpreterError::new_with_pos(
                "Type mismatch",
                args[0].location(),
            )),
        }
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
}
