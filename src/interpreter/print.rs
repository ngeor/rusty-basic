use crate::common::*;
use crate::instruction_generator::PrinterType;
use crate::interpreter::io::Printer;
use crate::variant::Variant;
use std::fmt::Display;

/// Handles the PRINT and LPRINT statements.
#[derive(Debug)]
pub struct PrintInterpreter {
    printer_type: PrinterType,
    file_handle: FileHandle,
    format_string: Option<String>,
    should_skip_new_line: bool,
    format_string_idx: usize,
}

impl PrintInterpreter {
    pub fn new() -> Self {
        Self {
            printer_type: PrinterType::Print,
            file_handle: 0.into(),
            format_string: None,
            should_skip_new_line: false,
            format_string_idx: 0,
        }
    }

    fn reset(&mut self) {
        self.printer_type = PrinterType::Print;
        self.file_handle = 0.into();
        self.format_string = None;
        self.format_string_idx = 0;
    }

    pub fn get_printer_type(&self) -> PrinterType {
        self.printer_type
    }

    pub fn set_printer_type(&mut self, printer_type: PrinterType) {
        self.reset();
        self.printer_type = printer_type;
    }

    pub fn get_file_handle(&self) -> FileHandle {
        self.file_handle
    }

    pub fn set_file_handle(&mut self, file_handle: FileHandle) {
        self.file_handle = file_handle;
    }

    pub fn set_format_string(&mut self, format_string: Option<String>) {
        self.format_string = format_string;
    }

    pub fn print_comma(&mut self, printer: Box<&dyn Printer>) -> std::io::Result<usize> {
        self.should_skip_new_line = true;
        printer.move_to_next_print_zone()
    }

    pub fn print_semicolon(&mut self) {
        self.should_skip_new_line = true;
    }

    pub fn print_value(&mut self, printer: Box<&dyn Printer>, v: Variant) -> Result<(), QError> {
        self.should_skip_new_line = false;
        if self.format_string.is_some() {
            self.print_value_with_format_string(printer, v)
        } else {
            self.print_value_without_format_string(printer, v)
                .map(|_| ())
                .map_err(QError::from)
        }
    }

    pub fn print_end(&mut self, printer: Box<&dyn Printer>) -> Result<(), QError> {
        if self.format_string.is_some() {
            self.print_remaining_chars(&printer)?;
        }
        if self.should_skip_new_line {
            self.should_skip_new_line = false;
            Ok(())
        } else {
            printer.println().map(|_| ()).map_err(QError::from)
        }
    }

    fn print_remaining_chars(&mut self, printer: &Box<&dyn Printer>) -> Result<(), QError> {
        let format_string_chars: Vec<char> = self.format_string.as_ref().unwrap().chars().collect();
        print_remaining_non_formatting_chars(
            printer,
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
        )
    }

    fn print_value_with_format_string(
        &mut self,
        printer: Box<&dyn Printer>,
        v: Variant,
    ) -> Result<(), QError> {
        let format_string_chars: Vec<char> = self.format_string.as_ref().unwrap().chars().collect();
        if format_string_chars.is_empty() {
            return Err(QError::IllegalFunctionCall);
        }

        // ensure we are in the range of chars
        self.format_string_idx = self.format_string_idx % format_string_chars.len();

        // copy from format_string until we hit a formatting character
        print_non_formatting_chars(
            &printer,
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
        )?;

        // format the argument using the formatting character
        print_formatting_chars(
            printer,
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
            v,
        )
    }

    fn print_value_without_format_string(
        &mut self,
        printer: Box<&dyn Printer>,
        v: Variant,
    ) -> std::io::Result<usize> {
        printer.print_variant(&v)
    }
}

fn print_non_formatting_chars(
    printer: &Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<(), QError> {
    // copy from format_string until we hit a formatting character
    let mut buf: String = String::new();
    let starting_index = *idx;
    let mut i = starting_index;
    while !is_formatting_char(format_string_chars[i]) {
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

fn is_formatting_char(ch: char) -> bool {
    ch == '#' || ch == '\\' || ch == '!'
}

fn print_remaining_non_formatting_chars(
    printer: &Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<(), QError> {
    // copy from format_string until we hit a formatting character
    let mut buf: String = String::new();
    let starting_index = *idx;
    let mut i = starting_index;
    while i < format_string_chars.len() && !is_formatting_char(format_string_chars[i]) {
        buf.push(format_string_chars[i]);
        i += 1;
    }
    printer.print(buf.as_str())?;
    *idx = i;
    Ok(())
}

fn print_formatting_chars(
    printer: Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<(), QError> {
    match format_string_chars[*idx] {
        '#' => print_digit_formatting_chars(printer, format_string_chars, idx, v),
        '\\' => print_string_formatting_chars(printer, format_string_chars, idx, v),
        '!' => print_first_char_formatting_chars(printer, format_string_chars, idx, v),
        _ => Err(QError::InternalError(format!(
            "Not a formatting character: {}",
            format_string_chars[*idx]
        ))),
    }
}

fn print_digit_formatting_chars(
    printer: Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<(), QError> {
    debug_assert_eq!(format_string_chars[*idx], '#');
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

fn print_string_formatting_chars(
    printer: Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<(), QError> {
    debug_assert_eq!(format_string_chars[*idx], '\\');
    *idx += 1;
    let mut counter: usize = 2;
    while *idx < format_string_chars.len() && format_string_chars[*idx] != '\\' {
        if format_string_chars[*idx] != ' ' {
            // only spaces should be allowed within backslashes
            return Err(QError::IllegalFunctionCall);
        }
        *idx += 1;
        counter += 1;
    }
    if *idx < format_string_chars.len() {
        *idx += 1;
        if let Variant::VString(s) = v {
            printer.print(s.fix_length(counter).as_str())?;
            Ok(())
        } else {
            Err(QError::TypeMismatch)
        }
    } else {
        // did not find closing backslash
        Err(QError::IllegalFunctionCall)
    }
}
fn print_first_char_formatting_chars(
    printer: Box<&dyn Printer>,
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<(), QError> {
    debug_assert_eq!(format_string_chars[*idx], '!');
    if let Variant::VString(s) = v {
        let ch = s.chars().next().ok_or(QError::IllegalFunctionCall)?;
        printer.print(String::from(ch).as_str())?;
        *idx += 1;
        Ok(())
    } else {
        Err(QError::TypeMismatch)
    }
}

trait PrintHelper {
    fn print_number<V: Display>(&self, number: V, leading_space: bool) -> std::io::Result<usize>;

    fn print_variant(&self, v: &Variant) -> std::io::Result<usize>;
}

impl<T: Printer + ?Sized> PrintHelper for T {
    fn print_number<V: Display>(&self, number: V, leading_space: bool) -> std::io::Result<usize> {
        let s: String = if leading_space {
            format!(" {} ", number)
        } else {
            format!("{} ", number)
        };
        self.print(s.as_str())
    }

    fn print_variant(&self, v: &Variant) -> std::io::Result<usize> {
        match v {
            Variant::VSingle(f) => self.print_number(f, *f >= 0.0),
            Variant::VDouble(d) => self.print_number(d, *d >= 0.0),
            Variant::VString(s) => self.print(s),
            Variant::VInteger(i) => self.print_number(i, *i >= 0),
            Variant::VLong(l) => self.print_number(l, *l >= 0),
            Variant::VArray(_) | Variant::VUserDefined(_) => panic!(
                "Cannot print user defined type {:?}, linter should have caught this",
                v
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_lprints_exact;
    use crate::assert_prints;
    use crate::assert_prints_exact;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

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
        assert_interpreter_err!("PRINT USING \"\"; 0", QError::IllegalFunctionCall, 1, 17);
    }

    #[test]
    fn test_print_using_without_format_specifiers_is_error() {
        assert_interpreter_err!(
            "PRINT USING \"oops\"; 12",
            QError::IllegalFunctionCall,
            1,
            21
        );
    }

    #[test]
    fn test_print_using_numeric_format_string_with_string_arg_is_error() {
        assert_interpreter_err!("PRINT USING \"#.##\"; \"hi\"", QError::TypeMismatch, 1, 21);
    }

    #[test]
    fn test_print_using_integer_format_string_with_string_arg_is_error() {
        assert_interpreter_err!("PRINT USING \"##\"; \"hi\"", QError::TypeMismatch, 1, 19);
    }

    #[test]
    fn test_print_zones_when_arg_contains_crlf() {
        let input = r#"
        X$ = CHR$(13) + CHR$(10)
        A$ = "hello, " + X$ + " world"
        PRINT "a", A$, "z"
        "#;
        assert_prints_exact!(input, "a             hello, ", "", " world        z", "");
    }

    #[test]
    fn test_print_using_backslash() {
        let input = r#"
        '            hello
        PRINT USING "\   \"; "hello world"
        "#;
        assert_prints!(input, "hello");
    }

    #[test]
    fn test_print_using_backslash_when_string_contains_nulls() {
        let input = r#"
        DIM A$
        A$ = "12" + CHR$(0)
        PRINT USING "\ \"; A$;
        "#;
        assert_prints_exact!(input, "12 ");
    }

    #[test]
    fn test_print_using_exclamation_point() {
        let input = r#"
        PRINT USING "!"; "hello world"
        "#;
        assert_prints!(input, "h");
    }
}
