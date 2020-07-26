// CHR$(ascii-code%) returns the text representation of the given ascii code

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::linter::{Error, ExpressionNode};

pub struct Chr {}

impl BuiltInLint for Chr {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        util::require_single_numeric_argument(args)
    }
}

impl BuiltInRun for Chr {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        _pos: Location,
    ) -> Result<(), InterpreterError> {
        let i: i32 = interpreter.pop_integer();
        let mut s: String = String::new();
        s.push((i as u8) as char);
        interpreter.function_result = s.into();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::linter::LinterError;

    #[test]
    fn test_chr() {
        assert_prints!("PRINT CHR$(33)", "!");
        assert_linter_err!(
            "PRINT CHR$(33, 34)",
            LinterError::ArgumentCountMismatch,
            1,
            7
        );
        assert_linter_err!(
            r#"PRINT CHR$("33")"#,
            LinterError::ArgumentTypeMismatch,
            1,
            12
        );
    }
}
