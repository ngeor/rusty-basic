use crate::common::{FileHandle, QError};
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::{PrintArg, PrintNode};
use crate::variant::Variant;
use std::convert::TryFrom;

pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QError> {
    let mut idx: usize = 0;
    let has_file = bool::try_from(interpreter.context().get(idx).unwrap())?;
    idx += 1;

    let file_handle: FileHandle = if has_file {
        FileHandle::try_from(interpreter.context().get(idx).unwrap())?
    } else {
        FileHandle::default()
    };

    if has_file {
        idx += 1;
    }

    let mut print_args: Vec<String> = vec![];
    while idx < interpreter.context().parameter_count() {
        let v = interpreter.context().get(idx).unwrap();
        idx += 1;
        print_args.push(v.to_string());
    }

    if file_handle.is_valid() {
        interpreter.file_manager.print(&file_handle, print_args)?;
    } else {
        interpreter.stdlib.print(print_args);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;

    #[test]
    fn test_print_no_args() {
        assert_prints!("PRINT", "");
    }

    #[test]
    fn test_interpret_print_hello_world_one_arg() {
        let input = "PRINT \"Hello, world!\"";
        assert_prints!(input, "Hello, world!");
    }

    #[test]
    fn test_interpret_print_hello_world_two_args() {
        let input = r#"PRINT "Hello", "world!""#;
        assert_prints!(input, "Hello world!");
    }

    #[test]
    fn test_interpret_print_hello_world_two_args_one_is_function() {
        let input = r#"
            PRINT "Hello", Test(1)
            FUNCTION Test(N)
                Test = N + 1
            END FUNCTION
            "#;
        assert_prints!(input, "Hello 2");
    }

    #[test]
    fn test_print_trailing_comma_does_not_add_new_line() {
        let input = r#"
            PRINT "123456789012345"
            PRINT "A",
            PRINT "B"
            "#;
        assert_prints!(input, "123456789012345", "A             B");
    }

    #[test]
    fn test_print_trailing_semicolon_does_not_add_new_line() {
        let input = r#"
            PRINT "A";
            PRINT "B"
            "#;
        assert_prints!(input, "AB");
    }
}
