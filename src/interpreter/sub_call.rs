use super::*;
use crate::common::Result;
use crate::parser::{Expression, QName, TypeQualifier};

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn sub_call(&mut self, name: &String, args: &Vec<Expression>) -> Result<()> {
        if name == "PRINT" {
            self._do_print(args)
        } else if name == "INPUT" {
            self._do_input(args)
        } else if name == "SYSTEM" {
            self.stdlib.system();
            Ok(())
        } else {
            self.err(format!("Unknown sub {}", name))
        }
    }

    fn _do_print(&mut self, args: &Vec<Expression>) -> Result<()> {
        let mut strings: Vec<String> = vec![];
        for a in args {
            strings.push(self._do_print_map_arg(a)?);
        }
        self.stdlib.print(strings);
        Ok(())
    }

    fn _do_print_map_arg(&mut self, arg: &Expression) -> Result<String> {
        let evaluated = self.evaluate_expression(arg)?;
        Ok(evaluated.to_string())
    }

    fn _do_input(&mut self, args: &Vec<Expression>) -> Result<()> {
        for a in args {
            self._do_input_one(a)?;
        }
        Ok(())
    }

    fn _do_input_one(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::VariableName(n) => self._do_input_one_var(n),
            _ => self.err(format!("Expected variable name, was {:?}", expression)),
        }
    }

    fn _do_input_one_var(&mut self, qualified_name: &QName) -> Result<()> {
        let raw_input: String = self.stdlib.input()?;
        let variable_value = match self.effective_type_qualifier(qualified_name) {
            TypeQualifier::BangSingle => Variant::from(parse_single_input(raw_input)?),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(parse_int_input(raw_input)?),
            _ => unimplemented!(),
        };
        self.set_variable(qualified_name, variable_value);
        Ok(())
    }
}

fn parse_single_input(s: String) -> Result<f32> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| format!("Could not parse {} as float: {}", s, e))
    }
}

fn parse_int_input(s: String) -> Result<i32> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| format!("Could not parse {} as int: {}", s, e))
    }
}

#[cfg(test)]
mod tests {
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
}
