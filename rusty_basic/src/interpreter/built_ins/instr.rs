use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::utils::VariantCasts;
use crate::parser::BuiltInFunction;
use crate::variant::Variant;
use rusty_common::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let a: &Variant = &interpreter.context()[0];
    let b: &Variant = &interpreter.context()[1];
    let result: i32 = match interpreter.context().variables().get(2) {
        Some(c) => do_instr(
            a.to_positive_int()?,
            b.to_str_unchecked(),
            c.to_str_unchecked(),
        )?,
        None => do_instr(1, a.to_str_unchecked(), b.to_str_unchecked())?,
    };
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::InStr, result);
    Ok(())
}

fn do_instr(start: usize, hay: &str, needle: &str) -> Result<i32, QError> {
    debug_assert!(start >= 1);
    if hay.is_empty() {
        Ok(0)
    } else if needle.is_empty() {
        Ok(1)
    } else {
        let mut i: usize = start - 1;
        while i + needle.len() <= hay.len() {
            let sub = hay.get(i..(i + needle.len())).unwrap();
            if sub == needle {
                return Ok((i as i32) + 1);
            }
            i += 1;
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::interpret_err;
    use rusty_common::*;

    #[test]
    fn test_instr_happy_flow() {
        assert_prints!(r#"PRINT INSTR("hay", "needle")"#, "0");
        assert_prints!(r#"PRINT INSTR("hay", "hay")"#, "1");
        assert_prints!(r#"PRINT INSTR("hay", "a")"#, "2");
        assert_prints!(r#"PRINT INSTR("hay", "z")"#, "0");
        assert_prints!(r#"PRINT INSTR("hello there", "the")"#, "7");
        assert_prints!(r#"PRINT INSTR(2, "the the", "the")"#, "5");
    }

    #[test]
    fn test_instr_edge_cases() {
        assert_prints!(r#"PRINT INSTR("hay", "")"#, "1");
        assert_prints!(r#"PRINT INSTR("", "needle")"#, "0");
        assert_prints!(r#"PRINT INSTR("", "")"#, "0");
        assert_eq!(
            interpret_err(r#"PRINT INSTR(0, "oops", "zero")"#),
            ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
        );
    }
}
