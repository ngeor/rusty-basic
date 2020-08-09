// VAL(str-expr$) converts a string representation of a number to a number.

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterErrorNode};
use crate::variant;
use crate::variant::Variant;

pub struct Val {}

impl BuiltInLint for Val {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        util::require_single_string_argument(args)
    }
}

impl BuiltInRun for Val {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VString(s) => val(s).with_err_no_pos()?,
            _ => panic!("unexpected arg to VAL"),
        };
        Ok(())
    }
}

fn val(s: String) -> Result<Variant, String> {
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
    use crate::assert_prints;

    #[test]
    fn test_val_float() {
        let program = r#"PRINT VAL("3.14")"#;
        assert_prints!(program, "3.14");
    }

    #[test]
    fn test_val_integer() {
        let program = r#"PRINT VAL("42")"#;
        assert_prints!(program, "42");
    }

    #[test]
    fn test_val_invalid_string_gives_zero() {
        let program = r#"PRINT VAL("oops")"#;
        assert_prints!(program, "0");
    }

    #[test]
    fn test_val_partial_parse() {
        let program = r#"PRINT VAL("3.14oops")"#;
        assert_prints!(program, "3.14");
    }

    #[test]
    fn test_val_partial_parse_ignores_spaces() {
        let program = r#"PRINT VAL("  -    4   . 2   ")"#;
        assert_prints!(program, "-4.2");
    }

    #[test]
    fn test_val_no_overflow() {
        let program = r#"PRINT VAL("1234567890123456789012345678901234567890")"#;
        assert_prints!(program, "1234567890123456800000000000000000000000");
    }
}
