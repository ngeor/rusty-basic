use crate::instruction_generator::PrinterType;
use crate::interpreter::io::Printer;
use rusty_common::*;
use rusty_variant::Variant;
use std::fmt::Display;

/// Handles the PRINT and LPRINT statements.
#[derive(Debug)]
pub struct PrintState {
    printer_type: PrinterType,
    file_handle: FileHandle,
    format_string: Option<String>,
    should_skip_new_line: bool,
    format_string_idx: usize,
}

impl PrintState {
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

    pub fn on_print_comma(&mut self) {
        self.should_skip_new_line = true;
    }

    pub fn print_semicolon(&mut self) {
        self.should_skip_new_line = true;
    }

    pub fn print_value_from_a(
        &mut self,
        v: Variant,
    ) -> Result<(Option<String>, Option<Variant>), QError> {
        self.should_skip_new_line = false;
        if self.format_string.is_some() {
            let s = self.print_value_with_format_string(v)?;
            Ok((Some(s), None))
        } else {
            Ok((None, Some(v)))
        }
    }

    pub fn print_end(&mut self) -> Result<(Option<String>, bool), QError> {
        let opt_remaining = if self.format_string.is_some() {
            Some(self.print_remaining_chars()?)
        } else {
            None
        };
        let should_print_new_line = if self.should_skip_new_line {
            self.should_skip_new_line = false;
            false
        } else {
            true
        };
        Ok((opt_remaining, should_print_new_line))
    }

    fn print_remaining_chars(&mut self) -> Result<String, QError> {
        let format_string_chars: Vec<char> = self.format_string.as_ref().unwrap().chars().collect();
        print_remaining_non_formatting_chars(
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
        )
    }

    fn print_value_with_format_string(&mut self, v: Variant) -> Result<String, QError> {
        let format_string_chars: Vec<char> = self.format_string.as_ref().unwrap().chars().collect();
        if format_string_chars.is_empty() {
            return Err(QError::IllegalFunctionCall);
        }

        // ensure we are in the range of chars
        self.format_string_idx %= format_string_chars.len();

        // copy from format_string until we hit a formatting character
        let mut result = print_non_formatting_chars(
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
        )?;

        // format the argument using the formatting character
        let second_part = print_formatting_chars(
            format_string_chars.as_slice(),
            &mut self.format_string_idx,
            v,
        )?;
        result.push_str(&second_part);

        Ok(result)
    }
}

fn print_non_formatting_chars(
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<String, QError> {
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
    *idx = i;
    Ok(buf)
}

fn is_formatting_char(ch: char) -> bool {
    ch == '#' || ch == '\\' || ch == '!'
}

fn print_remaining_non_formatting_chars(
    format_string_chars: &[char],
    idx: &mut usize,
) -> Result<String, QError> {
    // copy from format_string until we hit a formatting character
    let mut buf: String = String::new();
    let starting_index = *idx;
    let mut i = starting_index;
    while i < format_string_chars.len() && !is_formatting_char(format_string_chars[i]) {
        buf.push(format_string_chars[i]);
        i += 1;
    }
    *idx = i;
    Ok(buf)
}

fn print_formatting_chars(
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<String, QError> {
    match format_string_chars[*idx] {
        '#' => numeric_formatting::print_digit_formatting_chars(format_string_chars, idx, v),
        '\\' => print_string_formatting_chars(format_string_chars, idx, v),
        '!' => print_first_char_formatting_chars(format_string_chars, idx, v),
        _ => Err(QError::InternalError(format!(
            "Not a formatting character: {}",
            format_string_chars[*idx]
        ))),
    }
}

mod numeric_formatting {
    //! Handles formatting of numbers.

    use rusty_common::*;
    use rusty_variant::Variant;

    pub fn print_digit_formatting_chars(
        format_string_chars: &[char],
        idx: &mut usize,
        v: Variant,
    ) -> Result<String, QError> {
        debug_assert_eq!(format_string_chars[*idx], '#');
        // collect just the formatting chars e.g. ###,###.##
        let number_format_chars: &[char] = format_string_chars[*idx..]
            .split(|ch| *ch != '#' && *ch != ',' && *ch != '.')
            .next()
            .expect("Should find at least one formatting character");
        // increment the index
        *idx += number_format_chars.len();
        // split at decimal point
        let mut decimal_split = number_format_chars.split(|ch| *ch == '.');
        let integer_part = decimal_split.next().unwrap();
        if integer_part.is_empty() {
            // leading dot
            return Err(QError::IllegalFunctionCall);
        }
        (match decimal_split.next() {
            Some(fractional_part) => {
                if fractional_part.is_empty() {
                    // trailing dot
                    Err(QError::IllegalFunctionCall)
                } else {
                    fmt_with_fractional_part(integer_part, fractional_part, v)
                }
            }
            _ => fmt_without_fractional_part(integer_part, v),
        })
        .map_err(QError::from)
    }

    fn fmt_with_fractional_part(
        integer_fmt: &[char],
        fractional_fmt: &[char],
        v: Variant,
    ) -> Result<String, QError> {
        let unformatted = format_variant(v, fractional_fmt.len())?;
        let mut unformatted_decimal_split = unformatted.split('.');
        let unformatted_integer = unformatted_decimal_split.next().unwrap();
        let unformatted_fractional = unformatted_decimal_split.next().unwrap_or("");
        let mut result = fmt_integer_part(integer_fmt, unformatted_integer)?;

        // append the fractional parts
        result.push('.');
        for ch in unformatted_fractional
            .chars()
            .chain(std::iter::repeat('0'))
            .take(fractional_fmt.len())
        {
            result.push(ch);
        }
        Ok(result)
    }

    fn fmt_without_fractional_part(integer_fmt: &[char], v: Variant) -> Result<String, QError> {
        let unformatted: String = format_variant(v, 0)?;
        fmt_integer_part(integer_fmt, &unformatted)
    }

    fn fmt_integer_part(integer_fmt: &[char], unformatted_str: &str) -> Result<String, QError> {
        let mut result: String = String::new();
        let unformatted: Vec<char> = unformatted_str.chars().collect();
        // start with the rightmost digit
        let mut i: usize = integer_fmt.len();
        let mut j: usize = unformatted.len();
        while i > 0 || j > 0 {
            if i > 0 {
                match integer_fmt[i - 1] {
                    ',' => {
                        result.insert(0, if j > 0 { ',' } else { ' ' });
                    }
                    '#' => {
                        if j > 0 {
                            result.insert(0, unformatted[j - 1]);
                            j -= 1;
                        } else {
                            result.insert(0, ' ');
                        }
                    }
                    _ => {
                        // unsupported formatting character
                        return Err(QError::IllegalFunctionCall);
                    }
                }
                i -= 1;
            } else {
                // we run out formatting characters but we still have digits to print
                result.insert(0, unformatted[j - 1]);
                j -= 1;
            }
        }
        Ok(result)
    }

    fn format_variant(v: Variant, fractional_digits: usize) -> Result<String, QError> {
        match v {
            Variant::VSingle(f) => Ok(if fractional_digits > 0 {
                format!("{:.1$}", f, fractional_digits)
            } else {
                let l = f.round() as i64;
                l.to_string()
            }),
            Variant::VDouble(d) => Ok(if fractional_digits > 0 {
                format!("{:.1$}", d, fractional_digits)
            } else {
                let l = d.round() as i64;
                l.to_string()
            }),
            Variant::VInteger(i) => Ok(i.to_string()),
            Variant::VLong(l) => Ok(l.to_string()),
            _ => Err(QError::TypeMismatch),
        }
    }
}

fn print_string_formatting_chars(
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<String, QError> {
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
            Ok(s.fix_length(counter))
        } else {
            Err(QError::TypeMismatch)
        }
    } else {
        // did not find closing backslash
        Err(QError::IllegalFunctionCall)
    }
}

fn print_first_char_formatting_chars(
    format_string_chars: &[char],
    idx: &mut usize,
    v: Variant,
) -> Result<String, QError> {
    debug_assert_eq!(format_string_chars[*idx], '!');
    if let Variant::VString(s) = v {
        let ch = s.chars().next().ok_or(QError::IllegalFunctionCall)?;
        let result = String::from(ch);
        *idx += 1;
        Ok(result)
    } else {
        Err(QError::TypeMismatch)
    }
}

pub trait PrintHelper {
    fn print_number<V: Display>(
        &mut self,
        number: V,
        leading_space: bool,
    ) -> std::io::Result<usize>;

    fn print_variant(&mut self, v: &Variant) -> std::io::Result<usize>;
}

impl<T: Printer + ?Sized> PrintHelper for T {
    fn print_number<V: Display>(
        &mut self,
        number: V,
        leading_space: bool,
    ) -> std::io::Result<usize> {
        let s: String = if leading_space {
            format!(" {} ", number)
        } else {
            format!("{} ", number)
        };
        self.print(s.as_str())
    }

    fn print_variant(&mut self, v: &Variant) -> std::io::Result<usize> {
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
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use rusty_common::*;

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
    fn test_print_using_thousands_placeholder_no_decimals() {
        assert_prints_exact!("PRINT USING \"###,###\"; 1000;", "  1,000");
    }

    #[test]
    fn test_print_using_thousands_placeholder_no_decimals_less_than_thousand() {
        assert_prints_exact!("PRINT USING \"###,###\"; 42;", "     42");
    }

    #[test]
    fn test_print_using_thousands_placeholder_two_decimals() {
        assert_prints_exact!("PRINT USING \"###,###.##\"; 1000;", "  1,000.00");
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
            assert_prints_exact!(&program, output[i]);
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
            assert_prints_exact!(&program, output[i]);
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
