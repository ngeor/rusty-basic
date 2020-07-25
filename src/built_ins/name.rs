use crate::common::Location;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::lexer::Keyword;
use crate::linter::{err_l, err_no_pos, Error, ExpressionNode, LinterError};
use crate::parser::{BareName, Parser, ParserError, Statement, TypeQualifier};
use std::io::BufRead;

pub fn demand<T: BufRead>(parser: &mut Parser<T>) -> Result<Statement, ParserError> {
    parser.read_demand_whitespace("Expected space after NAME")?;
    let old_file_name = parser.read_demand_expression()?;
    parser.read_demand_whitespace("Expected space after filename")?;
    parser.read_demand_keyword(Keyword::As)?;
    parser.read_demand_whitespace("Expected space after AS")?;
    let new_file_name = parser.read_demand_expression()?;
    parser.read_demand_eol_or_eof_skipping_whitespace()?;
    let bare_name: BareName = "NAME".into();
    Ok(Statement::SubCall(
        bare_name,
        vec![old_file_name, new_file_name],
    ))
}

pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), Error> {
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

pub fn run<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    pos: Location,
) -> Result<(), InterpreterError> {
    let old_file_name = interpreter.pop_string();
    let new_file_name = interpreter.pop_string();
    std::fs::rename(old_file_name, new_file_name)
        .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))
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
