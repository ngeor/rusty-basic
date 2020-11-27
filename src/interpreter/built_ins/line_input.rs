// LINE INPUT -> see INPUT
// LINE INPUT [;] ["prompt";] variable$
// LINE INPUT #file-number%, variable$

use super::*;
use crate::interpreter::input::Input;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let mut file_handle: FileHandle = FileHandle::default();
    let mut has_file_handle = false;
    for idx in 0..interpreter.context().parameter_count() {
        let v = interpreter.context().get(idx).unwrap();
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
        .try_get_file_info_input_mut(file_handle)?;
    let s = file_input.line_input()?;
    *interpreter.context_mut().get_mut(idx).unwrap() = Variant::VString(s);
    Ok(())
}

fn line_input_one_stdin<S: InterpreterTrait>(
    interpreter: &mut S,
    idx: usize,
) -> Result<(), QError> {
    let s = interpreter.stdin().input()?;
    *interpreter.context_mut().get_mut(idx).unwrap() = Variant::VString(s);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_interpreter_err;

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
}
