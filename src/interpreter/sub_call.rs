use super::*;
use super::Interpreter;
use super::Stdlib;
use crate::common::Result;
use crate::parser::{Expression, QName, TypeQualifier};
use std::io::BufRead;

impl<T: BufRead, TStdlib: Stdlib> Interpreter<T, TStdlib> {
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
        Ok(evaluated.to_str())
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
            TypeQualifier::BangFloat => Variant::from(parse_float_input(raw_input)?),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(parse_int_input(raw_input)?),
            _ => unimplemented!(),
        };
        self.set_variable(qualified_name, variable_value)
    }
}

fn parse_float_input(s: String) -> Result<f32> {
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
            use crate::interpreter::*;
            use crate::interpreter::test_utils::*;
            use crate::parser::QName;

            fn test_input<S: AsRef<str>>(user_input: S) -> Variant {
                let mut stdlib = MockStdlib::new();
                stdlib.next_input = user_input.as_ref().to_string();
                let input = "INPUT N";
                let mut interpreter = Interpreter::new_from_bytes(input, stdlib);
                interpreter.interpret().unwrap();
                interpreter
                    .get_variable(&QName::Untyped("N".to_string()))
                    .unwrap()
            }

            #[test]
            fn test_input_empty() {
                assert_eq!(test_input(""), Variant::VFloat(0.0));
            }

            #[test]
            fn test_input_zero() {
                assert_eq!(test_input("0"), Variant::VFloat(0.0));
            }

            #[test]
            fn test_input_float() {
                assert_eq!(test_input("1.1"), Variant::VFloat(1.1));
            }

            #[test]
            fn test_input_negative() {
                assert_eq!(test_input("-1.2345"), Variant::VFloat(-1.2345));
            }

            #[test]
            fn test_input_explicit_positive() {
                assert_eq!(test_input("+3.14"), Variant::VFloat(3.14));
            }
        }

        mod string_var {
            use crate::interpreter::*;
            use crate::interpreter::test_utils::*;
            use crate::parser::{QName, TypeQualifier};

            fn test_input<S: AsRef<str>>(user_input: S) -> Variant {
                let mut stdlib = MockStdlib::new();
                stdlib.next_input = user_input.as_ref().to_string();
                let input = "INPUT A$";
                let mut interpreter = Interpreter::new_from_bytes(input, stdlib);
                interpreter.interpret().unwrap();
                interpreter
                    .get_variable(&QName::Typed("A".to_string(), TypeQualifier::DollarString))
                    .unwrap()
            }

            #[test]
            fn test_input_hello() {
                let var = test_input("hello");
                assert_eq!(var, Variant::from("hello"));
            }

            #[test]
            fn test_input_does_not_trim_new_line() {
                let var = test_input("hello\r\n");
                assert_eq!(var, Variant::from("hello\r\n"));
            }
        }

        mod int_var {
            use crate::interpreter::*;
            use crate::interpreter::test_utils::*;
            use crate::parser::{QName, TypeQualifier};

            fn test_input<S: AsRef<str>>(user_input: S) -> Variant {
                let mut stdlib = MockStdlib::new();
                stdlib.next_input = user_input.as_ref().to_string();
                let input = "INPUT A%";
                let mut interpreter = Interpreter::new_from_bytes(input, stdlib);
                interpreter.interpret().unwrap();
                interpreter
                    .get_variable(&QName::Typed("A".to_string(), TypeQualifier::PercentInteger))
                    .unwrap()
            }

            #[test]
            fn test_input_42() {
                let var = test_input("42");
                assert_eq!(var, Variant::from(42));
            }
        }
    }
}
