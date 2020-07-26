// MID$ function returns part of a string
// MID$ statement replaces part of a string (TODO support this)
// MID$(str_expr$, start%[, length%])
// MID$(str_var$, start%[, length%]) = str_expr$
// if the length is omitted, returns or replaces all remaining characters

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::linter::{err_no_pos, Error, ExpressionNode, LinterError};

pub struct Mid {}

impl BuiltInLint for Mid {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() == 2 {
            util::require_string_argument(args, 0)?;
            util::require_integer_argument(args, 1)
        } else if args.len() == 3 {
            util::require_string_argument(args, 0)?;
            util::require_integer_argument(args, 1)?;
            util::require_integer_argument(args, 2)
        } else {
            err_no_pos(LinterError::ArgumentCountMismatch)
        }
    }
}

impl BuiltInRun for Mid {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let s: String = interpreter.pop_string();
        let start: i32 = interpreter.pop_integer();
        let length: Option<i32> = interpreter.pop_unnamed_val().map(|v| v.demand_integer());
        let result: String = do_mid(s, start, length, pos)?;
        interpreter.function_result = result.into();
        Ok(())
    }
}

fn do_mid(
    s: String,
    start: i32,
    opt_length: Option<i32>,
    pos: Location,
) -> Result<String, InterpreterError> {
    if start <= 0 {
        Err(InterpreterError::new_with_pos("Illegal function call", pos))
    } else {
        let start_idx: usize = (start - 1) as usize;
        match opt_length {
            Some(length) => {
                if length < 0 {
                    Err(InterpreterError::new_with_pos("Illegal function call", pos))
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
    use crate::common::Location;
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
            InterpreterError::new(
                "Illegal function call",
                vec![
                    Location::new(1, 7),
                    Location::new(1, 7) // TODO why is this double
                ]
            )
        );
        assert_eq!(
            interpret_err(r#"PRINT MID$("hay", 1, -1)"#),
            InterpreterError::new(
                "Illegal function call",
                vec![
                    Location::new(1, 7),
                    Location::new(1, 7) // TODO why is this double
                ]
            )
        );
    }

    #[test]
    fn test_mid_linter() {
        assert_linter_err!("PRINT MID$()", LinterError::ArgumentCountMismatch, 1, 7);
        assert_linter_err!(
            r#"PRINT MID$("oops")"#,
            LinterError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
