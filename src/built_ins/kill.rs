// KILL file-spec$ -> deletes files from disk

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};

pub struct Kill {}

impl BuiltInRun for Kill {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_name = interpreter.pop_string();
        std::fs::remove_file(file_name)
            .map_err(|e| e.into())
            .with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_kill_happy_flow() {
        std::fs::write("KILL1.TXT", "hi").unwrap_or(());
        interpret(r#"KILL "KILL1.TXT""#);
        std::fs::read_to_string("KILL1.TXT").expect_err("File should have been deleted");
    }

    #[test]
    fn test_kill_edge_cases() {
        assert_eq!(
            interpret_err(r#"KILL "KILL2.TXT""#),
            ErrorEnvelope::Pos(QError::FileNotFound, Location::new(1, 1))
        );
    }

    #[test]
    fn test_kill_linter() {
        assert_linter_err!("KILL", QError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL "a", "b""#, QError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL 42"#, QError::ArgumentTypeMismatch, 1, 6);
    }
}
