use crate::common::FileHandle;
use crate::instruction_generator::print::PrintHandle;
use crate::interpreter::printer::Printer;
use crate::variant::Variant;
use std::fmt::Display;

#[derive(Debug)]
pub struct PrintInterpreter {
    print_handle: PrintHandle,
    file_handle: FileHandle,
    format_string: Option<String>,
    should_skip_new_line: bool,
}

impl PrintInterpreter {
    pub fn new() -> Self {
        Self {
            print_handle: PrintHandle::Print,
            file_handle: 0.into(),
            format_string: None,
            should_skip_new_line: false,
        }
    }

    pub fn reset(&mut self) {
        self.print_handle = PrintHandle::Print;
        self.file_handle = 0.into();
        self.format_string = None;
    }

    pub fn get_print_handle(&self) -> PrintHandle {
        self.print_handle
    }

    pub fn set_print_handle(&mut self, print_handle: PrintHandle) {
        self.print_handle = print_handle;
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

    pub fn print_value(
        &mut self,
        printer: Box<&dyn Printer>,
        v: Variant,
    ) -> std::io::Result<usize> {
        self.should_skip_new_line = false;
        printer.print_variant(&v)
    }

    pub fn print_end(&mut self, printer: Box<&dyn Printer>) -> std::io::Result<usize> {
        if self.should_skip_new_line {
            self.should_skip_new_line = false;
            Ok(0)
        } else {
            printer.println()
        }
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
        assert_interpreter_err!("PRINT USING \"\"; 0", QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn test_print_using_without_format_specifiers_is_error() {
        assert_interpreter_err!(
            "PRINT USING \"oops\"; 12",
            QError::IllegalFunctionCall,
            1,
            1
        );
    }

    #[test]
    fn test_print_using_numeric_format_string_with_string_arg_is_error() {
        assert_interpreter_err!("PRINT USING \"#.##\"; \"hi\"", QError::TypeMismatch, 1, 1);
    }

    #[test]
    fn test_print_using_integer_format_string_with_string_arg_is_error() {
        assert_interpreter_err!("PRINT USING \"##\"; \"hi\"", QError::TypeMismatch, 1, 1);
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
}
