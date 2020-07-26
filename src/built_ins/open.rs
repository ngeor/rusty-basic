// OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]
//
// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
// access: READ, WRITE, READ WRITE
// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
// file-number a number in the range 1 through 255 (TODO enforce this)
// rec-len%: For random access files, the record length (default is 128 bytes)
//           For sequential files, the number of characters buffered (default is 512 bytes)

use super::{BuiltInLint, BuiltInParse, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterError, Stdlib};
use crate::lexer::{Keyword, LexemeNode};
use crate::linter::{Error, ExpressionNode};
use crate::parser::{
    unexpected, BareName, Expression, Parser, ParserError, Statement, StatementContext,
    StatementNode,
};
use std::io::BufRead;

pub struct Open {}

impl BuiltInParse for Open {
    fn demand<T: BufRead>(
        &self,
        parser: &mut Parser<T>,
        pos: Location,
        _context: StatementContext,
    ) -> Result<StatementNode, ParserError> {
        parser.read_demand_whitespace("Expected space after OPEN")?;
        let file_name_expr = parser.read_demand_expression()?;
        parser.read_demand_whitespace("Expected space after filename")?;
        parser.read_demand_keyword(Keyword::For)?;
        parser.read_demand_whitespace("Expected space after FOR")?;
        let mode: i32 = read_demand_file_mode(parser)?.into();
        parser.read_demand_whitespace("Expected space after file mode")?;
        let mut next = parser.buf_lexer.read()?;
        let mut access: i32 = FileAccess::Unspecified.into();
        if next.is_keyword(Keyword::Access) {
            parser.read_demand_whitespace("Expected space after ACCESS")?;
            access = read_demand_file_access(parser)?.into();
            parser.read_demand_whitespace("Expected space after file access")?;
            next = parser.buf_lexer.read()?;
        }
        if next.is_keyword(Keyword::As) {
            parser.read_demand_whitespace("Expected space after AS")?;
            let file_handle = parser.read_demand_expression()?;
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
        } else {
            unexpected("Expected AS", next)
        }
    }
}

fn read_demand_file_mode<T: BufRead>(parser: &mut Parser<T>) -> Result<FileMode, ParserError> {
    let next = parser.buf_lexer.read()?;
    match next {
        LexemeNode::Keyword(Keyword::Input, _, _) => Ok(FileMode::Input),
        LexemeNode::Keyword(Keyword::Output, _, _) => Ok(FileMode::Output),
        LexemeNode::Keyword(Keyword::Append, _, _) => Ok(FileMode::Append),
        _ => unexpected("Expected INPUT|OUTPUT|APPEND after FOR", next),
    }
}

fn read_demand_file_access<T: BufRead>(parser: &mut Parser<T>) -> Result<FileAccess, ParserError> {
    let next = parser.buf_lexer.read()?;
    match next {
        LexemeNode::Keyword(Keyword::Read, _, _) => Ok(FileAccess::Read),
        _ => unexpected("Expected READ after ACCESS", next),
    }
}

impl BuiltInLint for Open {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), Error> {
        // not needed because of special parsing
        Ok(())
    }
}

impl BuiltInRun for Open {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let file_name = interpreter.pop_string();
        let file_mode: FileMode = interpreter.pop_integer().into();
        let file_access: FileAccess = interpreter.pop_integer().into();
        let file_handle = interpreter.pop_file_handle();
        interpreter
            .file_manager
            .open(file_handle, file_name.as_ref(), file_mode, file_access)
            .map_err(|e| {
                InterpreterError::new_with_pos(format!("Could not open {}: {}", file_name, e), pos)
            })
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
