// OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]
//
// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
// access: READ, WRITE, READ WRITE
// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
// file-number a number in the range 1 through 255 (TODO enforce this)
// rec-len%: For random access files, the record length (default is 128 bytes)
//           For sequential files, the number of characters buffered (default is 512 bytes)

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::lexer::{BufLexer, Keyword, Lexeme, LexemeNode};
use crate::linter::ExpressionNode;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::{BareName, Expression, Statement, StatementNode};
use std::io::BufRead;
#[derive(Debug)]
pub struct Open {}

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    let Locatable { element: next, pos } = lexer.peek()?;
    if !next.is_keyword(Keyword::Open) {
        return Ok(None);
    }

    lexer.read()?;
    read_demand_whitespace(lexer, "Expected space after OPEN")?;
    let file_name_expr = demand(lexer, expression::try_read, "Expected filename")?;
    read_demand_whitespace(lexer, "Expected space after filename")?;
    read_demand_keyword(lexer, Keyword::For)?;
    read_demand_whitespace(lexer, "Expected space after FOR")?;
    let mode: i32 = read_demand_file_mode(lexer)?.into();
    read_demand_whitespace(lexer, "Expected space after file mode")?;
    let mut next: LexemeNode = lexer.read()?;
    let mut access: i32 = FileAccess::Unspecified.into();
    if next.as_ref().is_keyword(Keyword::Access) {
        read_demand_whitespace(lexer, "Expected space after ACCESS")?;
        access = read_demand_file_access(lexer)?.into();
        read_demand_whitespace(lexer, "Expected space after file access")?;
        next = lexer.read()?;
    }
    if next.as_ref().is_keyword(Keyword::As) {
        read_demand_whitespace(lexer, "Expected space after AS")?;
        let file_handle = demand(lexer, expression::try_read, "Expected file handle")?;
        let bare_name: BareName = "OPEN".into();

        Ok(Statement::SubCall(
            bare_name,
            vec![
                file_name_expr,
                Expression::IntegerLiteral(mode).at(Location::start()),
                Expression::IntegerLiteral(access).at(Location::start()),
                file_handle,
            ],
        )
        .at(pos))
        .map(|x| Some(x))
    } else {
        Err(QError::SyntaxError("Expected AS".to_string())).with_err_at(&next)
    }
}

fn read_demand_file_mode<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<FileMode, QErrorNode> {
    let next = lexer.read()?;
    match next.as_ref() {
        Lexeme::Keyword(Keyword::Input, _) => Ok(FileMode::Input),
        Lexeme::Keyword(Keyword::Output, _) => Ok(FileMode::Output),
        Lexeme::Keyword(Keyword::Append, _) => Ok(FileMode::Append),
        _ => Err(QError::SyntaxError(
            "Expected INPUT|OUTPUT|APPEND after FOR".to_string(),
        ))
        .with_err_at(&next),
    }
}

fn read_demand_file_access<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<FileAccess, QErrorNode> {
    let next = lexer.read()?;
    match next.as_ref() {
        Lexeme::Keyword(Keyword::Read, _) => Ok(FileAccess::Read),
        _ => Err(QError::SyntaxError(
            "Expected READ after ACCESS".to_string(),
        ))
        .with_err_at(&next),
    }
}

impl BuiltInLint for Open {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // not needed because of special parsing
        Ok(())
    }
}

impl BuiltInRun for Open {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_name = interpreter.pop_string();
        let file_mode: FileMode = interpreter.pop_integer().into();
        let file_access: FileAccess = interpreter.pop_integer().into();
        let file_handle = interpreter.pop_file_handle();
        interpreter
            .file_manager
            .open(file_handle, file_name.as_ref(), file_mode, file_access)
            .map_err(|e| e.into())
            .with_err_no_pos()
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::test_utils::*;
    use crate::interpreter::DefaultStdlib;
    use crate::interpreter::Interpreter;

    #[test]
    fn test_can_create_file() {
        let input = r#"
        OPEN "TEST1.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        CLOSE #1
        "#;
        let instructions = generate_instructions(input);
        let mut interpreter = Interpreter::new(DefaultStdlib {});
        interpreter.interpret(instructions).unwrap_or_default();
        let contents = std::fs::read_to_string("TEST1.TXT").unwrap_or("".to_string());
        std::fs::remove_file("TEST1.TXT").unwrap_or(());
        assert_eq!("Hello, world\r\n", contents);
    }

    #[test]
    fn test_can_read_file() {
        let input = r#"
        OPEN "TEST2A.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        CLOSE #1
        OPEN "TEST2A.TXT" FOR INPUT AS #1
        LINE INPUT #1, T$
        CLOSE #1
        OPEN "TEST2B.TXT" FOR APPEND AS #1
        PRINT #1, T$
        CLOSE #1
        "#;
        let instructions = generate_instructions(input);
        let mut interpreter = Interpreter::new(DefaultStdlib {});
        interpreter.interpret(instructions).unwrap_or_default();
        let contents = std::fs::read_to_string("TEST2B.TXT").unwrap_or("".to_string());
        std::fs::remove_file("TEST2A.TXT").unwrap_or(());
        std::fs::remove_file("TEST2B.TXT").unwrap_or(());
        assert_eq!("Hello, world\r\n", contents);
    }

    #[test]
    fn test_can_read_file_until_eof() {
        let input = r#"
        OPEN "TEST3.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        PRINT #1, "Hello, again"
        CLOSE #1
        OPEN "TEST3.TXT" FOR INPUT AS #1
        WHILE NOT EOF(1)
        LINE INPUT #1, T$
        PRINT T$
        WEND
        CLOSE #1
        "#;
        let instructions = generate_instructions(input);
        let stdlib = MockStdlib::new();
        let mut interpreter = Interpreter::new(stdlib);
        interpreter.interpret(instructions).unwrap_or_default();
        std::fs::remove_file("TEST3.TXT").unwrap_or(());
        assert_eq!(
            interpreter.stdlib.output,
            vec!["Hello, world", "Hello, again"]
        );
    }
}
