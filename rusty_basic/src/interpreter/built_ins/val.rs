use rusty_parser::BuiltInFunction;
use rusty_variant::{MAX_INTEGER, MAX_LONG, Variant, VariantError};

use crate::RuntimeError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let v: &str = interpreter.context()[0].to_str_unchecked();
    let result: Variant = val(v)?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Val, result);
    Ok(())
}

fn val(s: &str) -> Result<Variant, VariantError> {
    let mut is_positive = true;
    let mut value: f64 = 0.0;
    let mut fraction_power: i32 = 0;

    const STATE_INITIAL: u8 = 0;
    const STATE_SIGN: u8 = 1;
    const STATE_INT: u8 = 2;
    const STATE_DOT: u8 = 3;
    const STATE_FRACTION: u8 = 4;
    let mut state: u8 = STATE_INITIAL;

    for c in s.chars() {
        if ('0'..='9').contains(&c) {
            if state == STATE_INITIAL || state == STATE_SIGN {
                state = STATE_INT;
            } else if state == STATE_DOT {
                state = STATE_FRACTION;
            }
            if state == STATE_INT {
                value = value * 10.0 + ((c as u8) - b'0') as f64;
            } else {
                if fraction_power <= MAX_INTEGER {
                    fraction_power += 1;
                } else {
                    return Err(VariantError::Overflow);
                }
                value = (value * 10.0_f64.powi(fraction_power) + ((c as u8) - b'0') as f64)
                    / 10.0_f64.powi(fraction_power);
            }
        } else if c == ' ' {
            // ignore spaces apparently
        } else if c == '.' {
            if state == STATE_DOT || state == STATE_FRACTION {
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
        if is_positive && value <= MAX_INTEGER as f64 {
            Ok(Variant::VInteger(value as i32))
        } else if !is_positive && value <= (1 + MAX_INTEGER) as f64 {
            Ok(Variant::VInteger(-value as i32))
        } else if is_positive && value <= MAX_LONG as f64 {
            Ok(Variant::VLong(value as i64))
        } else if !is_positive && value <= (1 + MAX_LONG) as f64 {
            Ok(Variant::VLong(-value as i64))
        } else {
            let x = Variant::VDouble(value);
            if is_positive { Ok(x) } else { x.negate() }
        }
    } else {
        let x = Variant::VDouble(value);
        if is_positive { Ok(x) } else { x.negate() }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

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
