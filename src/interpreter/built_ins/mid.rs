// MID$ function returns part of a string
// MID$ statement replaces part of a string (TODO support this)
// MID$(str_expr$, start%[, length%])
// MID$(str_var$, start%[, length%]) = str_expr$
// if the length is omitted, returns or replaces all remaining characters
use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let s: &String = (&interpreter.context()[0]).try_into().with_err_no_pos()?;
    let start: i32 = (&interpreter.context()[1]).try_into().with_err_no_pos()?;
    let length: Option<i32> = match interpreter.context().variables().get(2) {
        Some(v) => Some(v.try_into().with_err_no_pos()?),
        None => None,
    };
    let result: String = do_mid(s, start, length)?;
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Mid.into(), result.into());
    Ok(())
}

fn do_mid(s: &String, start: i32, opt_length: Option<i32>) -> Result<String, QErrorNode> {
    if start <= 0 {
        Err(QError::IllegalFunctionCall).with_err_no_pos()
    } else {
        let start_idx: usize = (start - 1) as usize;
        match opt_length {
            Some(length) => {
                if length < 0 {
                    Err(QError::IllegalFunctionCall).with_err_no_pos()
                } else {
                    let end: usize = if start_idx + (length as usize) > s.len() {
                        s.len()
                    } else {
                        start_idx + (length as usize)
                    };
                    Ok(s.get(start_idx..end).unwrap_or_default().to_string())
                }
            }
            None => Ok(s.get(start_idx..).unwrap_or_default().to_string()),
        }
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
    fn test_mid_happy_flow() {
        assert_prints!(r#"PRINT MID$("hay", 1)"#, "hay");
        assert_prints!(r#"PRINT MID$("hay", 2)"#, "ay");
        assert_prints!(r#"PRINT MID$("hay", 1, 1)"#, "h");
        assert_prints!(r#"PRINT MID$("hay", 2, 2)"#, "ay");
        assert_prints!(r#"PRINT MID$("hay", 2, 20)"#, "ay");
    }

    #[test]
    fn test_mid_edge_cases() {
        assert_prints!(r#"PRINT MID$("", 1)"#, "");
        assert_prints!(r#"PRINT MID$("hay", 4)"#, "");
        assert_prints!(r#"PRINT MID$("hay", 1, 0)"#, "");
        assert_eq!(
            interpret_err(r#"PRINT MID$("hay", 0)"#),
            ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
        );
        assert_eq!(
            interpret_err(r#"PRINT MID$("hay", 1, -1)"#),
            ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
        );
    }

    #[test]
    fn test_mid_linter() {
        assert_linter_err!(r#"PRINT MID$("oops")"#, QError::ArgumentCountMismatch, 1, 7);
    }
}
