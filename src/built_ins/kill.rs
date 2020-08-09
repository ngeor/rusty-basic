// KILL file-spec$ -> deletes files from disk

use super::{util, BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterErrorNode};

pub struct Kill {}

impl BuiltInLint for Kill {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        util::require_single_string_argument(args)
    }
}

impl BuiltInRun for Kill {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        let file_name = interpreter.pop_string();
        std::fs::remove_file(file_name)
            .map_err(|e| map_err(e))
            .with_err_no_pos()
    }
}

fn map_err(e: std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::NotFound {
        "File not found".to_string()
    } else {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;
    use crate::interpreter::test_utils::*;
    use crate::linter::LinterError;

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
            ErrorEnvelope::Stacktrace(
                "File not found".to_string(),
                vec![
                    Location::new(1, 1),
                    Location::new(1, 1) // TODO why is this double
                ]
            )
        );
    }

    #[test]
    fn test_kill_linter() {
        assert_linter_err!("KILL", LinterError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL "a", "b""#, LinterError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL 42"#, LinterError::ArgumentTypeMismatch, 1, 6);
    }
}
