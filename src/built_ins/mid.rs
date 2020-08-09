// MID$ function returns part of a string
// MID$ statement replaces part of a string (TODO support this)
// MID$(str_expr$, start%[, length%])
// MID$(str_var$, start%[, length%]) = str_expr$
// if the length is omitted, returns or replaces all remaining characters

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterError, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterError, LinterErrorNode};

pub struct Mid {}

impl BuiltInLint for Mid {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        if args.len() == 2 {
            util::require_string_argument(args, 0)?;
            util::require_integer_argument(args, 1)
        } else if args.len() == 3 {
            util::require_string_argument(args, 0)?;
            util::require_integer_argument(args, 1)?;
            util::require_integer_argument(args, 2)
        } else {
            Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

impl BuiltInRun for Mid {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        let s: String = interpreter.pop_string();
        let start: i32 = interpreter.pop_integer();
        let length: Option<i32> = interpreter.pop_unnamed_val().map(|v| v.demand_integer());
        let result: String = do_mid(s, start, length)?;
        interpreter.function_result = result.into();
        Ok(())
    }
}

fn do_mid(s: String, start: i32, opt_length: Option<i32>) -> Result<String, InterpreterErrorNode> {
    if start <= 0 {
        Err(InterpreterError::IllegalFunctionCall).with_err_no_pos()
    } else {
        let start_idx: usize = (start - 1) as usize;
        match opt_length {
            Some(length) => {
                if length < 0 {
                    Err(InterpreterError::IllegalFunctionCall).with_err_no_pos()
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
    use crate::interpreter::test_utils::interpret_err;
    use crate::interpreter::InterpreterError;
    use crate::linter::LinterError;

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
            ErrorEnvelope::Stacktrace(
                InterpreterError::IllegalFunctionCall,
                vec![
                    Location::new(1, 7),
                    Location::new(1, 7) // TODO why is this double
                ]
            )
        );
        assert_eq!(
            interpret_err(r#"PRINT MID$("hay", 1, -1)"#),
            ErrorEnvelope::Stacktrace(
                InterpreterError::IllegalFunctionCall,
                vec![
                    Location::new(1, 7),
                    Location::new(1, 7) // TODO why is this double
                ]
            )
        );
    }

    #[test]
    fn test_mid_linter() {
        assert_linter_err!(
            r#"PRINT MID$("oops")"#,
            LinterError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
