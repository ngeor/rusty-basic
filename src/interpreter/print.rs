use crate::common::{FileHandle, QError};
use crate::instruction_generator::print::{FLAG_COMMA, FLAG_EXPRESSION, FLAG_SEMICOLON};
use crate::interpreter::{Interpreter, PrintVal, Stdlib};
use std::convert::{TryFrom, TryInto};

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
        idx += 2; // skip file + skip comma after file
    }

    let mut print_val: PrintVal = PrintVal::NewLine;

    while idx < interpreter.context().parameter_count() {
        let v_type: u8 = interpreter.context().get(idx).unwrap().try_into()?;
        idx += 1;

        print_val = if v_type == FLAG_EXPRESSION {
            let v = interpreter.context().get(idx).unwrap();
            idx += 1;
            PrintVal::Value(v.clone())
        } else if v_type == FLAG_COMMA {
            PrintVal::Comma
        } else if v_type == FLAG_SEMICOLON {
            PrintVal::Semicolon
        } else {
            panic!("Unexpected PrintArg {}", v_type);
        };

        if file_handle.is_valid() {
            print_val.print(
                interpreter
                    .file_manager
                    .get_file_info_mut(&file_handle)
                    .unwrap(),
            )?;
        } else {
            print_val.print(&mut interpreter.stdlib)?;
        }
    }

    // print new line?
    match print_val {
        PrintVal::NewLine | PrintVal::Value(_) => {
            print_val = PrintVal::NewLine;
            if file_handle.is_valid() {
                print_val.print(
                    interpreter
                        .file_manager
                        .get_file_info_mut(&file_handle)
                        .unwrap(),
                )?;
            } else {
                print_val.print(&mut interpreter.stdlib)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_lprints;
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
    fn test_interpret_print_hello_world_two_args_comma() {
        let input = r#"PRINT "Hello", "world!""#;
        assert_prints!(input, "Hello         world!");
    }

    #[test]
    fn test_interpret_print_hello_world_two_args_semicolon() {
        let input = r#"PRINT "Hello, "; "world!""#;
        assert_prints!(input, "Hello, world!");
    }

    #[test]
    fn test_interpret_print_hello_world_two_args_one_is_function() {
        let input = r#"
            PRINT "Hello", Test(1)
            FUNCTION Test(N)
                Test = N + 1
            END FUNCTION
            "#;
        assert_prints!(input, "Hello         2");
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

    #[test]
    fn test_print_using() {
        assert_prints!("PRINT USING \"#.###\"; 3.14", "3.140");
    }

    #[test]
    fn test_lprint() {
        assert_lprints!("LPRINT 42", "42");
    }

    #[test]
    fn test_lprint_using() {
        assert_lprints!("LPRINT USING \"#.###\"; 3.14", "3.140");
    }
}
