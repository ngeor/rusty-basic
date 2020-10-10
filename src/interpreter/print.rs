use crate::common::{FileHandle, QError};
use crate::instruction_generator::print::{PrintArgType, PrintHandle};
use crate::interpreter::printer::Printer;
use crate::interpreter::{Interpreter, Stdlib};
use crate::variant::Variant;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;

enum PrintVal {
    Comma,
    Semicolon,
    NewLine,
    Value(Variant),
}

fn print_number<T: Printer, V: Display>(
    printer: &mut T,
    number: V,
    leading_space: bool,
) -> std::io::Result<usize> {
    let s: String = if leading_space {
        format!(" {} ", number)
    } else {
        format!("{} ", number)
    };
    printer.print(s.as_str())
}

impl PrintVal {
    fn print<T: Printer>(&self, printer: &mut T) -> std::io::Result<usize> {
        match self {
            Self::Comma => printer.move_to_next_print_zone(),
            Self::Semicolon => Ok(0),
            Self::NewLine => printer.println(),
            Self::Value(v) => match v {
                Variant::VSingle(f) => print_number(printer, f, *f >= 0.0),
                Variant::VDouble(d) => print_number(printer, d, *d >= 0.0),
                Variant::VString(s) => printer.print(s),
                Variant::VInteger(i) => print_number(printer, i, *i >= 0),
                Variant::VLong(l) => print_number(printer, l, *l >= 0),
                _ => panic!("Cannot print user defined type, linter should have caught this"),
            },
        }
    }
}

pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QError> {
    let mut idx: usize = 0;
    let output_type: PrintHandle = interpreter.context().get(idx).unwrap().try_into()?;
    idx += 1;

    let file_handle: FileHandle = if let PrintHandle::File = output_type {
        FileHandle::try_from(interpreter.context().get(idx).unwrap())?
    } else {
        FileHandle::default()
    };

    if file_handle.is_valid() {
        idx += 2; // skip file + skip comma after file
    }

    let mut print_val: PrintVal = PrintVal::NewLine;

    while idx < interpreter.context().parameter_count() {
        let v_type: PrintArgType = interpreter.context().get(idx).unwrap().try_into()?;
        idx += 1;

        print_val = match v_type {
            PrintArgType::Expression => {
                let v = interpreter.context().get(idx).unwrap();
                idx += 1;
                PrintVal::Value(v.clone())
            }
            PrintArgType::Comma => PrintVal::Comma,
            PrintArgType::Semicolon => PrintVal::Semicolon,
        };

        match output_type {
            PrintHandle::File => {
                print_val.print(
                    interpreter
                        .file_manager
                        .try_get_file_info_output_mut(&file_handle)
                        .unwrap(),
                )?;
            }
            PrintHandle::LPrint => {
                print_val.print(interpreter.stdlib.lpt1())?;
            }
            PrintHandle::Print => {
                print_val.print(&mut interpreter.stdlib)?;
            }
        }
    }

    // print new line?
    match print_val {
        PrintVal::NewLine | PrintVal::Value(_) => match output_type {
            PrintHandle::File => {
                interpreter
                    .file_manager
                    .try_get_file_info_output_mut(&file_handle)
                    .unwrap()
                    .println()?;
            }
            PrintHandle::LPrint => {
                interpreter.stdlib.lpt1().println()?;
            }
            PrintHandle::Print => {
                interpreter.stdlib.println()?;
            }
        },
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_lprints_exact;
    use crate::assert_prints;
    use crate::assert_prints_exact;

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
        assert_prints!(input, "Hello          2");
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
        assert_lprints_exact!("LPRINT 42", " 42 ", "");
    }

    #[test]
    fn test_lprint_using() {
        assert_lprints_exact!("LPRINT USING \"#.###\"; 3.14", " 3.140 ", "");
    }

    #[test]
    fn test_print_zones_numbers() {
        let input = r#"
        PRINT "1", "2", "3"
        PRINT 1, 2, 3
        PRINT -1, -2, -3
        "#;
        assert_prints_exact!(
            input,
            "1             2             3",
            " 1             2             3 ",
            "-1            -2            -3 ",
            ""
        );
    }
}
