use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;
use rusty_parser::specific::FileHandle;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let file_handles: Vec<FileHandle> = interpreter
        .context()
        .variables()
        .iter()
        .map(VariantCasts::to_file_handle)
        .collect::<Result<Vec<FileHandle>, RuntimeError>>()?;
    if file_handles.is_empty() {
        interpreter.file_manager().close_all();
    } else {
        for file_handle in file_handles {
            interpreter.file_manager().close(&file_handle);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_close_not_opened_file_is_allowed() {
        interpret("CLOSE 1");
    }

    #[test]
    fn test_close_allows_to_open_again() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        CLOSE #1
        OPEN "a.txt" FOR OUTPUT AS #1
        CLOSE #1
        "#;
        interpret(input);
        std::fs::remove_file("a.txt").unwrap_or(());
    }

    #[test]
    fn test_close_without_args_closes_all_files() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "b.txt" FOR OUTPUT AS #2
        CLOSE
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "b.txt" FOR OUTPUT AS #2
        CLOSE
        "#;
        interpret(input);
        std::fs::remove_file("a.txt").unwrap_or(());
        std::fs::remove_file("b.txt").unwrap_or(());
    }
}
