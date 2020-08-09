// NAME old$ AS new$
// Renames a file or directory.
// TODO support directory

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::lexer::{BufLexer, Keyword};
use crate::linter::ExpressionNode;
use crate::parser::buf_lexer::*;
use crate::parser::expression;
use crate::parser::{BareName, ParserErrorNode, Statement, StatementNode, TypeQualifier};
use std::io::BufRead;

#[derive(Debug)]
pub struct Name {}

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, ParserErrorNode> {
    let Locatable { element: next, pos } = lexer.peek()?;
    if next.is_keyword(Keyword::Name) {
        lexer.read()?;
        read_demand_whitespace(lexer, "Expected space after NAME")?;
        let old_file_name = demand(lexer, expression::try_read, "Expected original filename")?;
        read_demand_whitespace(lexer, "Expected space after filename")?;
        read_demand_keyword(lexer, Keyword::As)?;
        read_demand_whitespace(lexer, "Expected space after AS")?;
        let new_file_name = demand(lexer, expression::try_read, "Expected new filename")?;
        let bare_name: BareName = "NAME".into();
        Ok(Statement::SubCall(bare_name, vec![old_file_name, new_file_name]).at(pos))
            .map(|x| Some(x))
    } else {
        Ok(None)
    }
}

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
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
    }
}
