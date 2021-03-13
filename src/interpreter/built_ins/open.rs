// OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]
//
// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
// access: READ, WRITE, READ WRITE
// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
// file-number a number in the range 1 through 255
// rec-len%: For random access files, the record length (default is 128 bytes)
//           For sequential files, the number of characters buffered (default is 512 bytes)

use super::*;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let file_name: String = (&interpreter.context()[0]).to_string();
    let file_mode: FileMode = u8::try_from(&interpreter.context()[1])
        .with_err_no_pos()?
        .into();
    let file_access: FileAccess = u8::try_from(&interpreter.context()[2])
        .with_err_no_pos()?
        .into();
    let file_handle: FileHandle = (&interpreter.context()[3]).try_into().with_err_no_pos()?;
    let rec_len: i32 = (&interpreter.context()[4]).try_into().with_err_no_pos()?;
    interpreter
        .file_manager()
        .open(
            file_handle,
            &file_name,
            file_mode,
            file_access,
            rec_len as usize,
        )
        .with_err_no_pos()
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::*;

    fn read_and_remove(filename: &str) -> String {
        let contents = std::fs::read_to_string(filename).unwrap_or_default();
        std::fs::remove_file(filename).unwrap_or_default();
        contents
    }

    #[test]
    fn test_can_create_file() {
        std::fs::remove_file("TEST1.TXT").unwrap_or(());
        let input = r#"
        OPEN "TEST1.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        CLOSE #1
        "#;
        interpret(input);
        let contents = read_and_remove("TEST1.TXT");
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
        interpret(input);
        let contents = read_and_remove("TEST2B.TXT");
        std::fs::remove_file("TEST2A.TXT").unwrap_or(());
        std::fs::remove_file("TEST2B.TXT").unwrap_or(());
        assert_eq!("Hello, world\r\n", contents);
    }

    #[test]
    fn test_can_read_file_until_eof() {
        std::fs::remove_file("TEST3.TXT").unwrap_or(());
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
        assert_prints!(input, "Hello, world", "Hello, again");
        std::fs::remove_file("TEST3.TXT").unwrap_or(());
    }

    #[test]
    fn test_can_write_file_append_mode() {
        std::fs::remove_file("test_can_write_file_append_mode.TXT").unwrap_or(());
        let input = r#"
        OPEN "test_can_write_file_append_mode.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        PRINT #1, "Hello, again"
        CLOSE #1
        "#;
        interpret(input);
        let read_result = read_and_remove("test_can_write_file_append_mode.TXT");
        assert_eq!(read_result, "Hello, world\r\nHello, again\r\n");
    }

    #[test]
    fn test_open_bad_file_number_runtime_error() {
        let input = r#"
        A = 256
        OPEN "TEST.TXT" FOR INPUT AS A
        CLOSE A
        "#;
        assert_interpreter_err!(input, QError::BadFileNameOrNumber, 3, 9);
    }

    #[test]
    fn test_open_twice_without_closing_error() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "a.txt" FOR OUTPUT AS #1
        "#;
        assert_interpreter_err!(input, QError::FileAlreadyOpen, 3, 9);
        std::fs::remove_file("a.txt").unwrap_or(());
    }

    #[test]
    fn open_random_file_field_lset_put() {
        let input = r#"
        OPEN "rnd1.txt" FOR RANDOM AS #1 LEN = 64
        FIELD #1, 10 AS FirstName$, 20 AS LastName$
        LSET FirstName$ = "Nikos"
        LSET LastName$ = "Georgiou"
        PUT #1, 1
        CLOSE
        "#;
        interpret(input);
        let contents = read_and_remove("rnd1.txt");
        assert_eq!(contents, "Nikos\0\0\0\0\0Georgiou\0\0\0\0\0\0\0\0\0\0\0\0");
    }

    #[test]
    fn open_random_file_field_lset_put_get() {
        let input = r#"
        OPEN "rnd2.txt" FOR RANDOM AS #1 LEN = 15
        FIELD #1, 10 AS FirstName$, 5 AS LastName$
        LSET FirstName$ = "Nikos"
        LSET LastName$ = "Georgiou"
        PUT #1, 1
        LSET FirstName$ = "Someone"
        LSET LastName$ = "Else"
        PUT #1, 2
        GET #1, 1
        PRINT FirstName$; LastName$
        CLOSE
        "#;
        assert_prints!(input, "NikosGeorg");
        let contents = read_and_remove("rnd2.txt");
        assert_eq!(contents, "Nikos\0\0\0\0\0GeorgSomeone\0\0\0Else\0");
    }
}
