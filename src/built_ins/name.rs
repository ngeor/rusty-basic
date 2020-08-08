// NAME old$ AS new$
// Renames a file or directory.
// TODO support directory

use super::{BuiltInLint, BuiltInRun};
use crate::common::{AtLocation, HasLocation, Location};
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::lexer::{BufLexer, Keyword};
use crate::linter::{err_l, err_no_pos, Error, ExpressionNode, LinterError};
use crate::parser::buf_lexer::*;
use crate::parser::expression;
use crate::parser::{BareName, ParserError, Statement, StatementNode, TypeQualifier};
use std::io::BufRead;

#[derive(Debug)]
pub struct Name {}

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    let next = lexer.peek()?;
    if next.is_keyword(Keyword::Name) {
        lexer.read()?;
        let pos = next.pos();
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
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 2 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else if args[0].try_qualifier()? != TypeQualifier::DollarString {
            err_l(LinterError::ArgumentTypeMismatch, &args[0])
        } else if args[1].try_qualifier()? != TypeQualifier::DollarString {
            err_l(LinterError::ArgumentTypeMismatch, &args[1])
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for Name {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let old_file_name = interpreter.pop_string();
        let new_file_name = interpreter.pop_string();
        std::fs::rename(old_file_name, new_file_name)
            .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::interpreter::test_utils::*;
    use crate::linter::LinterError;

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
        assert_linter_err!(
            r#"NAME 1 AS "boo""#,
            LinterError::ArgumentTypeMismatch,
            1,
            6
        );
        assert_linter_err!(
            r#"NAME "boo" AS 2"#,
            LinterError::ArgumentTypeMismatch,
            1,
            15
        );
    }
}
