use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Input;
use rusty_common::{FileHandle, QError};
use rusty_variant::Variant;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let mut file_handle: FileHandle = FileHandle::default();
    let mut has_file_handle = false;
    for idx in 0..interpreter.context().variables().len() {
        let v = &interpreter.context()[idx];
        match v {
            Variant::VInteger(f) => {
                if idx == 0 {
                    has_file_handle = *f == 1;
                } else if idx == 1 {
                    if has_file_handle {
                        file_handle = FileHandle::try_from(*f)?;
                    } else {
                        // input integer variable?
                        panic!("Linter should have caught this");
                    }
                } else {
                    panic!("Linter should have caught this");
                }
            }
            Variant::VString(_) => {
                line_input_one(interpreter, idx, &file_handle)?;
            }
            _ => panic!("Linter should have caught this"),
        }
    }

    Ok(())
}

fn line_input_one<S: InterpreterTrait>(
    interpreter: &mut S,
    idx: usize,
    file_handle: &FileHandle,
) -> Result<(), QError> {
    if file_handle.is_valid() {
        line_input_one_file(interpreter, idx, file_handle)
    } else {
        line_input_one_stdin(interpreter, idx)
    }
}

fn line_input_one_file<S: InterpreterTrait>(
    interpreter: &mut S,
    idx: usize,
    file_handle: &FileHandle,
) -> Result<(), QError> {
    let file_input = interpreter
        .file_manager()
        .try_get_file_info_input(file_handle)?;
    let s = file_input.line_input()?;
    interpreter.context_mut()[idx] = Variant::VString(s);
    Ok(())
}

fn line_input_one_stdin<S: InterpreterTrait>(
    interpreter: &mut S,
    idx: usize,
) -> Result<(), QError> {
    let s = interpreter.stdin().input()?;
    interpreter.context_mut()[idx] = Variant::VString(s);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use rusty_common::*;

    #[test]
    fn test_line_input_string_from_file_eof() {
        std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
        std::fs::write(
            "test_line_input_string_from_file_eof.txt",
            "Hello\r\nWorld\r\n",
        )
        .unwrap();

        let input = r#"
        OPEN "test_line_input_string_from_file_eof.txt" FOR INPUT AS #1
        LINE INPUT #1, A$
        LINE INPUT #1, A$
        LINE INPUT #1, A$ ' should EOF here
        CLOSE
        "#;

        assert_interpreter_err!(input, QError::InputPastEndOfFile, 5, 9);
        std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
    }

    #[test]
    fn line_input_reading_into_array_user_defined_type_string() {
        let filename = "line_input_reading_into_array_user_defined_type_string.txt";
        std::fs::remove_file(filename).unwrap_or_default();
        std::fs::write(filename, "Hello world!!!\r\n").unwrap();
        let input = format!(
            r#"
            TYPE MyType
                Greeting AS STRING * 11
            END TYPE

            DIM A(1 TO 2) AS MyType

            OPEN "{}" FOR INPUT AS #1
            LINE INPUT #1, A(1).Greeting
            CLOSE

            PRINT A(1).Greeting
            "#,
            filename
        );
        assert_prints!(&input, "Hello world");
        std::fs::remove_file(filename).unwrap_or_default();
    }
}
