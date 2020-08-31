// CHR$(ascii-code%) returns the text representation of the given ascii code

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};

pub struct Chr {}

impl BuiltInRun for Chr {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
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
    use crate::common::QError;

    #[test]
    fn test_chr() {
        assert_prints!("PRINT CHR$(33)", "!");
        assert_linter_err!("PRINT CHR$(33, 34)", QError::ArgumentCountMismatch, 1, 7);
        assert_linter_err!(r#"PRINT CHR$("33")"#, QError::ArgumentTypeMismatch, 1, 12);
    }
}
