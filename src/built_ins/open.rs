use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::ExpressionNode;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::{Expression, Keyword, Statement};
use std::io::BufRead;

// OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]
// OpenStatement ::= OPEN<ws+><ExpressionNode>(<ws+>FOR mode)?(<ws+>ACCESS access)?<ws+>AS<ws+>[#]<file-number%>
//
// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
// access: READ, WRITE, READ WRITE
// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
// file-number a number in the range 1 through 255 (TODO enforce this)
// rec-len%: For random access files, the record length (default is 128 bytes)
//           For sequential files, the number of characters buffered (default is 512 bytes)

#[derive(Debug)]
pub struct Open {}

pub fn parse_open<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        crate::parser::pc::ws::seq2(
            opt_seq3(parse_filename(), parse_open_mode(), parse_open_access()),
            demand(
                parse_file_number(),
                QError::syntax_error_fn("Expected: AS file-number"),
            ),
            QError::syntax_error_fn("Expected: whitespace before AS"),
        ),
        |((file_name, opt_file_mode, opt_file_access), file_number)| {
            Statement::SubCall(
                "OPEN".into(),
                vec![
                    file_name,
                    // TODO take actual pos
                    Expression::IntegerLiteral(opt_file_mode.unwrap_or(FileMode::Input).into())
                        .at(Location::start()),
                    // TODO take actual pos
                    Expression::IntegerLiteral(
                        opt_file_access.unwrap_or(FileAccess::Unspecified).into(),
                    )
                    .at(Location::start()),
                    file_number,
                ],
            )
        },
    )
}

fn parse_filename<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<crate::parser::ExpressionNode, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        try_read_keyword(Keyword::Open),
        demand(
            expression::expression_node(),
            QError::syntax_error_fn("Expected: filename after OPEN"),
        ),
        QError::syntax_error_fn("Expected: whitespace after OPEN"),
    ))
}

fn parse_open_mode<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<FileMode, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::For)),
        demand(
            and_then(read_any_keyword(), |(k, _)| match k {
                Keyword::Append => Ok(FileMode::Append),
                Keyword::Input => Ok(FileMode::Input),
                Keyword::Output => Ok(FileMode::Output),
                _ => Err(QError::syntax_error("Invalid file mode")),
            }),
            QError::syntax_error_fn("Invalid file mode"),
        ),
        QError::syntax_error_fn("Expected: whitespace after FOR"),
    ))
}

fn parse_open_access<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<FileAccess, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::Access)),
        demand(
            and_then(read_any_keyword(), |(k, _)| match k {
                Keyword::Read => Ok(FileAccess::Read),
                _ => Err(QError::syntax_error("Invalid file access")),
            }),
            QError::syntax_error_fn("Invalid file access"),
        ),
        QError::syntax_error_fn("Expected: whitespace after ACCESS"),
    ))
}

fn parse_file_number<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<crate::parser::ExpressionNode, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        try_read_keyword(Keyword::As),
        demand(
            expression::expression_node(),
            QError::syntax_error_fn("Expected: file number"),
        ),
        QError::syntax_error_fn("Expected: whitespace after AS"),
    ))
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
