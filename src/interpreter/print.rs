use crate::common::{FileHandle, QError, StringUtils};
use crate::instruction_generator::print::{PrintArgType, PrintHandle};
use crate::interpreter::printer::Printer;
use crate::interpreter::{Interpreter, Stdlib};
use crate::variant::Variant;
use std::collections::VecDeque;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;

pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QError> {
    let parameter_count = interpreter.context().parameter_count();
    // get all args (cloned) to fight the borrow checker
    let mut args: VecDeque<Variant> = (0..parameter_count)
        .map(|i| interpreter.context().get(i).unwrap().clone())
        .collect();
    let (print_handle, file_handle) = decode_print_handle(&mut args)?;
    let opt_format_string: Option<String> = decode_format_string(&mut args)?.clone();
    let mut printer = PrinterWrapper {
        interpreter,
        print_handle,
        file_handle,
    };
    match opt_format_string {
        Some(format_string) => print_with_format_string(&mut printer, &mut args, format_string),
        _ => print_without_format_string(&mut printer, &mut args),
    }
}

fn decode_print_handle(args: &mut VecDeque<Variant>) -> Result<(PrintHandle, FileHandle), QError> {
    let print_handle = PrintHandle::try_from(
        args.pop_front()
            .expect("Should have print handle parameter"),
    )?;
    let file_handle = if let PrintHandle::File = print_handle {
        FileHandle::try_from(args.pop_front().expect("Should have file handle parameter"))?
    } else {
        FileHandle::default()
    };
    Ok((print_handle, file_handle))
}

fn decode_format_string(args: &mut VecDeque<Variant>) -> Result<Option<String>, QError> {
    let has_format_string: bool = args
        .pop_front()
        .expect("Expected flag parameter for format string")
        .try_into()?;
    if has_format_string {
        let v: Variant = args.pop_front().expect("Expected format string parameter");
        v.try_into().map(|x| Some(x))
    } else {
        Ok(None)
    }
}

fn decode_print_arg(args: &mut VecDeque<Variant>) -> Result<PrintVal, QError> {
    let print_arg_type =
        PrintArgType::try_from(args.pop_front().expect("Expected print arg type parameter"))?;
    match print_arg_type {
        PrintArgType::Expression => {
            let v = args.pop_front().expect("Expected expression parameter");
            Ok(PrintVal::Value(v))
        }
        PrintArgType::Comma => Ok(PrintVal::Comma),
        PrintArgType::Semicolon => Ok(PrintVal::Semicolon),
    }
}

struct PrinterWrapper<'a, S: Stdlib> {
    interpreter: &'a mut Interpreter<S>,
    print_handle: PrintHandle,
    file_handle: FileHandle,
}

impl<'a, S: Stdlib> Printer for PrinterWrapper<'a, S> {
    fn print(&mut self, s: &str) -> std::io::Result<usize> {
        match self.print_handle {
            PrintHandle::File => self
                .interpreter
                .file_manager
                .try_get_file_info_output_mut(&self.file_handle)
                .expect("Expected file handle")
                .print(s),
            PrintHandle::LPrint => self.interpreter.stdlib.lpt1().print(s),
            PrintHandle::Print => self.interpreter.stdlib.print(s),
        }
    }

    fn println(&mut self) -> std::io::Result<usize> {
        match self.print_handle {
            PrintHandle::File => self
                .interpreter
                .file_manager
                .try_get_file_info_output_mut(&self.file_handle)
                .expect("Expected file handle")
                .println(),
            PrintHandle::LPrint => self.interpreter.stdlib.lpt1().println(),
            PrintHandle::Print => self.interpreter.stdlib.println(),
        }
    }

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize> {
        match self.print_handle {
            PrintHandle::File => self
                .interpreter
                .file_manager
                .try_get_file_info_output_mut(&self.file_handle)
                .expect("Expected file handle")
                .move_to_next_print_zone(),
            PrintHandle::LPrint => self.interpreter.stdlib.lpt1().move_to_next_print_zone(),
            PrintHandle::Print => self.interpreter.stdlib.move_to_next_print_zone(),
        }
    }
}

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

fn print_without_format_string<T: Printer>(
    printer: &mut T,
    args: &mut VecDeque<Variant>,
) -> Result<(), QError> {
    let mut print_val: PrintVal = PrintVal::NewLine;

    while !args.is_empty() {
        print_val = decode_print_arg(args)?;
        print_val.print(printer)?;
    }

    // print new line?
    match print_val {
        PrintVal::NewLine | PrintVal::Value(_) => {
            printer.println()?;
        }
        _ => {}
    }
    Ok(())
}

fn print_with_format_string<T: Printer>(
    printer: &mut T,
    args: &mut VecDeque<Variant>,
    format_string: String,
) -> Result<(), QError> {
    let mut print_new_line = true;
    let format_string_chars: Vec<char> = format_string.chars().collect();
    if format_string_chars.is_empty() {
        return Err(QError::IllegalFunctionCall);
    }
    let mut format_string_idx: usize = 0;

    while !args.is_empty() {
        print_new_line = false;
        match decode_print_arg(args)? {
            PrintVal::Comma => {
                printer.move_to_next_print_zone()?;
            }
            PrintVal::Value(v) => {
                print_new_line = true;

                // copy from format_string until we hit a formatting character
                format_string_idx = format_string_idx % format_string_chars.len();
                print_non_formatting_chars(
                    printer,
                    format_string_chars.as_slice(),
                    &mut format_string_idx,
                )?;

                // format the argument using the formatting character
                print_formatting_chars(
                    printer,
                    format_string_chars.as_slice(),
                    &mut format_string_idx,
                    v,
                )?;
            }
            _ => {}
        }
    }

    // copy from format_string until we hit a formatting character
    print_remaining_non_formatting_chars(
        printer,
        format_string_chars.as_slice(),
        &mut format_string_idx,
    )?;

    // print new line?
    if print_new_line {
        printer.println()?;
    }
    Ok(())
}

fn print_non_formatting_chars<T: Printer>(
    printer: &mut T,
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<(), QError> {
    // copy from format_string until we hit a formatting character
    let mut buf: String = String::new();
    let starting_index = *idx;
    let mut i = starting_index;
    while format_string_chars[i] != '#' {
        buf.push(format_string_chars[i]);
        i = (i + 1) % format_string_chars.len();
        if i == starting_index {
            // looped over to the starting point without encountering a formatting character
            return Err(QError::IllegalFunctionCall);
        }
    }
    printer.print(buf.as_str())?;
    *idx = i;
    Ok(())
}

fn print_remaining_non_formatting_chars<T: Printer>(
    printer: &mut T,
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<(), QError> {
    // copy from format_string until we hit a formatting character
    let mut buf: String = String::new();
    let starting_index = *idx;
    let mut i = starting_index;
    while i < format_string_chars.len() && format_string_chars[i] != '#' {
        buf.push(format_string_chars[i]);
        i += 1;
    }
    printer.print(buf.as_str())?;
    *idx = i;
    Ok(())
}

fn print_formatting_chars<T: Printer>(
    printer: &mut T,
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<(), QError> {
    let mut integer_digits: usize = 0;
    while *idx < format_string_chars.len() && format_string_chars[*idx] == '#' {
        *idx += 1;
        integer_digits += 1;
    }
    if *idx < format_string_chars.len() && format_string_chars[*idx] == '.' {
        // it has a fractional part too
        *idx += 1;
        let mut fraction_digits: usize = 0;
        while *idx < format_string_chars.len() && format_string_chars[*idx] == '#' {
            *idx += 1;
            fraction_digits += 1;
        }
        // print dot
        // print decimal part with rounding
        match v {
            Variant::VSingle(f) => {
                // formatting to a variable precision with rounding https://stackoverflow.com/a/61101531/153258
                let mut x = format!("{:.1$}", f, fraction_digits);
                if let Some(dot_index) = x.find('.') {
                    let mut spaces_to_add: i32 = integer_digits as i32 - dot_index as i32;
                    while spaces_to_add > 0 {
                        x.insert(0, ' ');
                        spaces_to_add -= 1;
                    }
                }
                printer.print(x.as_str())?;
            }
            Variant::VDouble(d) => {
                let mut x = format!("{:.1$}", d, fraction_digits);
                if let Some(dot_index) = x.find('.') {
                    let mut spaces_to_add: i32 = integer_digits as i32 - dot_index as i32;
                    while spaces_to_add > 0 {
                        x.insert(0, ' ');
                        spaces_to_add -= 1;
                    }
                }
                printer.print(x.as_str())?;
            }
            Variant::VInteger(i) => {
                let mut x = i.to_string().pad_left(integer_digits);
                x.push('.');
                for _i in 0..fraction_digits {
                    x.push('0');
                }
                printer.print(x.as_str())?;
            }
            Variant::VLong(l) => {
                let mut x = l.to_string().pad_left(integer_digits);
                x.push('.');
                for _i in 0..fraction_digits {
                    x.push('0');
                }
                printer.print(x.as_str())?;
            }
            _ => {
                return Err(QError::TypeMismatch);
            }
        }
    } else {
        // just the integer part
        match v {
            Variant::VSingle(f) => {
                let l = f.round() as i64;
                let x = l.to_string().pad_left(integer_digits);
                printer.print(x.as_str())?;
            }
            Variant::VDouble(d) => {
                let l = d.round() as i64;
                let x = l.to_string().pad_left(integer_digits);
                printer.print(x.as_str())?;
            }
            Variant::VInteger(i) => {
                let x = i.to_string().pad_left(integer_digits);
                printer.print(x.as_str())?;
            }
            Variant::VLong(l) => {
                let x = l.to_string().pad_left(integer_digits);
                printer.print(x.as_str())?;
            }
            _ => {
                return Err(QError::TypeMismatch);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_err;
    use crate::assert_lprints_exact;
    use crate::assert_prints;
    use crate::assert_prints_exact;
    use crate::common::QError;

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
        assert_prints_exact!("PRINT USING \"#.###\"; 3.14", "3.140", "");
    }

    #[test]
    fn test_print_using_one_placeholder_two_variables() {
        assert_prints_exact!("PRINT USING \"####.##\"; 42; 3.147", "  42.00   3.15", "");
    }

    #[test]
    fn test_print_using_two_placeholders_two_variables() {
        assert_prints_exact!(
            "PRINT USING \"Income: ####.## Expense: ####.##\"; 42; 3.144",
            "Income:   42.00 Expense:    3.14",
            ""
        );
    }

    #[test]
    fn test_print_using_two_placeholders_one_variable() {
        assert_prints_exact!(
            "PRINT USING \"Income: ####.## Expense: ####.## omitted\"; 42",
            "Income:   42.00 Expense: ",
            ""
        );
    }

    #[test]
    fn test_print_using_two_placeholders_three_variables() {
        assert_prints_exact!(
            "PRINT USING \"A: # B: # C\"; 1; 2; 3",
            "A: 1 B: 2 CA: 3 B: ",
            ""
        );
    }

    #[test]
    fn test_lprint() {
        assert_lprints_exact!("LPRINT 42", " 42 ", "");
    }

    #[test]
    fn test_lprint_using() {
        assert_lprints_exact!("LPRINT USING \"#.###\"; 3.14", "3.140", "");
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

    #[test]
    fn test_print_using_numeric_types_with_two_integer_digits_format_string() {
        let input = ["42", "2", "-1", "3.0", "3.14#", "3.9", "61.9", "&HFFFFFF"];
        let output = ["42", " 2", "-1", " 3", " 3", " 4", "62", "16777215"];
        for i in 0..input.len() {
            let program = format!("PRINT USING \"##\"; {};", input[i]);
            assert_prints_exact!(program, output[i]);
        }
    }

    #[test]
    fn test_print_using_numeric_types_with_two_integer_digits_and_two_fraction_digits_format_string(
    ) {
        let input = [
            "42", "2", "-1", "3.0", "3.14#", "3.9", "61.9", "&HFFFFFF", "1.2345", "1.9876",
            "2.134#", "2.199#",
        ];
        let output = [
            "42.00",
            " 2.00",
            "-1.00",
            " 3.00",
            " 3.14",
            " 3.90",
            "61.90",
            "16777215.00",
            " 1.23",
            " 1.99",
            " 2.13",
            " 2.20",
        ];
        for i in 0..input.len() {
            let program = format!("PRINT USING \"##.##\"; {};", input[i]);
            assert_prints_exact!(program, output[i]);
        }
    }

    #[test]
    fn test_print_using_empty_format_string_is_error() {
        assert_err!("PRINT USING \"\"; 0", QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn test_print_using_without_format_specifiers_is_error() {
        assert_err!(
            "PRINT USING \"oops\"; 12",
            QError::IllegalFunctionCall,
            1,
            1
        );
    }

    #[test]
    fn test_print_using_numeric_format_string_with_string_arg_is_error() {
        assert_err!("PRINT USING \"#.##\"; \"hi\"", QError::TypeMismatch, 1, 1);
    }

    #[test]
    fn test_print_using_integer_format_string_with_string_arg_is_error() {
        assert_err!("PRINT USING \"##\"; \"hi\"", QError::TypeMismatch, 1, 1);
    }
}
