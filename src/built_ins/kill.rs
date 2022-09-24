pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_string_argument()
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let file_name: &str = interpreter.context()[0].to_str_unchecked();
        std::fs::remove_file(file_name).map_err(QError::from)
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
