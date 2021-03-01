// INSTR([start%,] hay$, needle$)
// if start% is omitted, INSTR starts at position 1
// returns the first occurrence of needle$ inside hay$

use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let a: &Variant = &interpreter.context()[0];
    let b: &Variant = &interpreter.context()[1];
    let result: i32 = match interpreter.context().variables().get(2) {
        Some(c) => do_instr(
            a.try_into().with_err_no_pos()?,
            b.try_into().with_err_no_pos()?,
            c.try_into().with_err_no_pos()?,
        )?,
        None => do_instr(
            1,
            a.try_into().with_err_no_pos()?,
            b.try_into().with_err_no_pos()?,
        )?,
    };
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::InStr.into(), result.into());
    Ok(())
}

fn do_instr(start: i32, hay: &String, needle: &String) -> Result<i32, QErrorNode> {
    if start <= 0 {
        Err(QError::IllegalFunctionCall).with_err_no_pos()
    } else if hay.is_empty() {
        Ok(0)
    } else if needle.is_empty() {
        Ok(1)
    } else {
        let mut i: usize = (start - 1) as usize;
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
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::interpret_err;

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

    #[test]
    fn test_instr_linter() {
        assert_linter_err!(
            r#"PRINT INSTR("oops")"#,
            QError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
