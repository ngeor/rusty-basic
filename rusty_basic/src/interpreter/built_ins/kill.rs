use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let file_name: &str = interpreter.context()[0].to_str_unchecked();
    std::fs::remove_file(file_name).map_err(RuntimeError::from)
}

#[cfg(test)]
mod tests {
    use crate::interpreter::test_utils::*;
    use crate::{ErrorEnvelope, RuntimeError};
    use rusty_common::*;

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
            ErrorEnvelope::Pos(RuntimeError::FileNotFound, Position::new(1, 1))
        );
    }
}
