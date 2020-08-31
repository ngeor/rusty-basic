use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::ExpressionNode;
use crate::parser::{ TypeQualifier};

// NAME old$ AS new$
// Renames a file or directory.
// TODO support directory

#[derive(Debug)]
pub struct Name {}

impl BuiltInLint for Name {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 2 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else if args[0].try_qualifier()? != TypeQualifier::DollarString {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else if args[1].try_qualifier()? != TypeQualifier::DollarString {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for Name {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let old_file_name = interpreter.pop_string();
        let new_file_name = interpreter.pop_string();
        std::fs::rename(old_file_name, new_file_name)
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
    fn test_can_rename_file() {
        // arrange
        std::fs::write("TEST4.OLD", "hi").unwrap_or(());
        let input = r#"
        NAME "TEST4.OLD" AS "TEST4.NEW"
        "#;
        // act
        interpret(input);
        // assert
        let contents = std::fs::read_to_string("TEST4.NEW").unwrap_or("".to_string());
        std::fs::remove_file("TEST4.OLD").unwrap_or(());
        std::fs::remove_file("TEST4.NEW").unwrap_or(());
        assert_eq!(contents, "hi");
    }

    #[test]
    fn test_can_rename_file_parenthesis() {
        // arrange
        std::fs::write("TEST5.OLD", "hi").unwrap_or(());
        let input = r#"
        NAME("TEST5.OLD")AS("TEST5.NEW")
        "#;
        // act
        interpret(input);
        // assert
        let contents = std::fs::read_to_string("TEST5.NEW").unwrap_or("".to_string());
        std::fs::remove_file("TEST5.OLD").unwrap_or(());
        std::fs::remove_file("TEST5.NEW").unwrap_or(());
        assert_eq!(contents, "hi");
    }

    #[test]
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
    }
}
