use crate::common::*;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, InterpreterError, Result, Stdlib};
use crate::linter::BuiltInFunction;
use crate::variant;
use crate::variant::Variant;
use std::convert::TryInto;

impl<S: Stdlib> Interpreter<S> {
    pub fn run_built_in_function(
        &mut self,
        function_name: &BuiltInFunction,
        pos: Location,
    ) -> Result<()> {
        match function_name {
            BuiltInFunction::Environ => self.run_environ(),
            BuiltInFunction::Len => self.run_len(),
            BuiltInFunction::Str => self.run_str(),
            BuiltInFunction::Val => self
                .run_val()
                .map_err(|e| InterpreterError::new_with_pos(e, pos)),
        }
    }

    fn run_environ(&mut self) -> Result<()> {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        match v {
            Variant::VString(env_var_name) => {
                let result = self.stdlib.get_env_var(&env_var_name);
                self.function_result = Variant::VString(result);
                Ok(())
            }
            _ => panic!("Type mismatch at ENVIRON$",),
        }
    }

    fn run_len(&mut self) -> Result<()> {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        self.function_result = match v {
            Variant::VSingle(_) => Variant::VInteger(4),
            Variant::VDouble(_) => Variant::VInteger(8),
            Variant::VString(v) => Variant::VInteger(v.len().try_into().unwrap()),
            Variant::VInteger(_) => Variant::VInteger(2),
            Variant::VLong(_) => Variant::VInteger(4),
        };
        Ok(())
    }

    fn run_str(&mut self) -> Result<()> {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        self.function_result = match v {
            Variant::VSingle(f) => Variant::VString(format!("{}", f)),
            Variant::VDouble(f) => Variant::VString(format!("{}", f)),
            Variant::VString(_) => panic!("unexpected arg to STR$"),
            Variant::VInteger(f) => Variant::VString(format!("{}", f)),
            Variant::VLong(f) => Variant::VString(format!("{}", f)),
        };
        Ok(())
    }

    fn run_val(&mut self) -> std::result::Result<(), String> {
        let v = self.context_mut().demand_sub().pop_front_unnamed();
        self.function_result = match v {
            Variant::VString(s) => val(s)?,
            _ => panic!("unexpected arg to VAL"),
        };
        Ok(())
    }
}

fn val(s: String) -> std::result::Result<Variant, String> {
    let mut is_positive = true;
    let mut value: f64 = 0.0;
    let mut frac_power: i32 = 0;

    const STATE_INITIAL: u8 = 0;
    const STATE_SIGN: u8 = 1;
    const STATE_INT: u8 = 2;
    const STATE_DOT: u8 = 3;
    const STATE_FRAC: u8 = 4;
    let mut state: u8 = STATE_INITIAL;

    for c in s.chars() {
        if c >= '0' && c <= '9' {
            if state == STATE_INITIAL || state == STATE_SIGN {
                state = STATE_INT;
            } else if state == STATE_DOT {
                state = STATE_FRAC;
            }
            if state == STATE_INT {
                value = value * 10.0 + ((c as u8) - ('0' as u8)) as f64;
            } else {
                if frac_power <= variant::MAX_INTEGER {
                    frac_power += 1;
                } else {
                    return Err("Overflow".to_string());
                }
                value = (value * 10.0_f64.powi(frac_power) + ((c as u8) - ('0' as u8)) as f64)
                    / 10.0_f64.powi(frac_power);
            }
        } else if c == ' ' {
            // ignore spaces apparently
        } else if c == '.' {
            if state == STATE_DOT || state == STATE_FRAC {
                break;
            } else {
                state = STATE_DOT;
            }
        } else if c == '-' {
            if state == STATE_INITIAL {
                state = STATE_SIGN;
                is_positive = false;
            } else {
                break;
            }
        } else if c == '+' {
            if state == STATE_INITIAL {
                state = STATE_SIGN;
            } else {
                break;
            }
        } else {
            // bail out
            break;
        }
    }

    if state == STATE_INITIAL || state == STATE_SIGN {
        Ok(Variant::VInteger(0))
    } else if state == STATE_INT || state == STATE_DOT {
        if is_positive && value <= variant::MAX_INTEGER as f64 {
            Ok(Variant::VInteger(value as i32))
        } else if !is_positive && value <= (1 + variant::MAX_INTEGER) as f64 {
            Ok(Variant::VInteger(-value as i32))
        } else if is_positive && value <= variant::MAX_LONG as f64 {
            Ok(Variant::VLong(value as i64))
        } else if !is_positive && value <= (1 + variant::MAX_LONG) as f64 {
            Ok(Variant::VLong(-value as i64))
        } else {
            let x = Variant::VDouble(value);
            if is_positive {
                Ok(x)
            } else {
                x.negate()
            }
        }
    } else {
        let x = Variant::VDouble(value);
        if is_positive {
            Ok(x)
        } else {
            x.negate()
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

    mod str_function {
        use super::*;

        #[test]
        fn test_str_float() {
            let program = r#"PRINT STR$(3.14)"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }
    }

    mod val {
        use super::*;

        #[test]
        fn test_val_float() {
            let program = r#"PRINT VAL("3.14")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }

        #[test]
        fn test_val_integer() {
            let program = r#"PRINT VAL("42")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42"]);
        }

        #[test]
        fn test_val_invalid_string_gives_zero() {
            let program = r#"PRINT VAL("oops")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["0"]);
        }

        #[test]
        fn test_val_partial_parse() {
            let program = r#"PRINT VAL("3.14oops")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }

        #[test]
        fn test_val_partial_parse_ignores_spaces() {
            let program = r#"PRINT VAL("  -    4   . 2   ")"#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["-4.2"]);
        }

        #[test]
        fn test_val_no_overflow() {
            let program = r#"PRINT VAL("1234567890123456789012345678901234567890")"#;
            let interpreter = interpret(program);
            assert_eq!(
                interpreter.stdlib.output,
                vec!["1234567890123456800000000000000000000000"]
            );
        }
    }
}
