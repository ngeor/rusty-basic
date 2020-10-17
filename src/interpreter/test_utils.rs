use crate::common::*;
use crate::instruction_generator::generate_instructions;
use crate::instruction_generator::test_utils::generate_instructions_str_with_types;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::stdlib::Stdlib;
use crate::interpreter::write_printer::WritePrinter;
use crate::linter;
use crate::linter::{DimName, UserDefinedTypes};
use crate::parser::parse_main_file;
use crate::variant::Variant;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub type MockStdout = WritePrinter<Vec<u8>>;
pub type MockInterpreter =
    Interpreter<MockStdlib, ReadInputSource<MockStdin>, MockStdout, MockStdout>;

pub fn mock_interpreter(user_defined_types: UserDefinedTypes) -> MockInterpreter {
    let stdlib = MockStdlib::new();
    let stdin = ReadInputSource::new(MockStdin { stdin: vec![] });
    let stdout = WritePrinter::new(vec![]);
    let lpt1 = WritePrinter::new(vec![]);
    Interpreter::new(stdlib, stdin, stdout, lpt1, user_defined_types)
}

pub fn interpret<T>(input: T) -> MockInterpreter
where
    T: AsRef<[u8]> + 'static,
{
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = mock_interpreter(user_defined_types);
    interpreter
        .interpret(instructions)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_with_raw_input<T>(input: T, raw_input: &str) -> MockInterpreter
where
    T: AsRef<[u8]> + 'static,
{
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = mock_interpreter(user_defined_types);
    if !raw_input.is_empty() {
        interpreter.stdin().add_next_input(raw_input);
    }
    interpreter
        .interpret(instructions)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_with_env<T, F>(input: T, mut initializer: F) -> MockInterpreter
where
    T: AsRef<[u8]> + 'static,
    F: FnMut(&mut MockInterpreter) -> (),
{
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = mock_interpreter(user_defined_types);
    initializer(&mut interpreter);
    interpreter
        .interpret(instructions)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_err<T>(input: T) -> QErrorNode
where
    T: AsRef<[u8]> + 'static,
{
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    let mut interpreter = mock_interpreter(user_defined_types);
    interpreter.interpret(instructions).unwrap_err()
}

pub fn interpret_file<S>(filename: S) -> Result<MockInterpreter, QErrorNode>
where
    S: AsRef<str>,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    let program = parse_main_file(f).unwrap();
    let (linted_program, user_defined_types) = linter::lint(program).unwrap();
    let instructions = generate_instructions(linted_program);
    let mut interpreter = mock_interpreter(user_defined_types);
    interpreter.interpret(instructions).map(|_| interpreter)
}

pub fn interpret_file_with_raw_input<S>(
    filename: S,
    raw_input: &str,
) -> Result<MockInterpreter, QErrorNode>
where
    S: AsRef<str>,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    let program = parse_main_file(f).unwrap();
    let (linted_program, user_defined_types) = linter::lint(program).unwrap();
    let instructions = generate_instructions(linted_program);
    let mut interpreter = mock_interpreter(user_defined_types);
    if !raw_input.is_empty() {
        interpreter.stdin().add_next_input(raw_input);
    }
    interpreter.interpret(instructions).map(|_| interpreter)
}

pub struct MockStdlib {
    pub env: HashMap<String, String>,
}

pub struct MockStdin {
    pub stdin: Vec<u8>,
}

impl ReadInputSource<MockStdin> {
    pub fn add_next_input<S: AsRef<str>>(&mut self, value: S) {
        self.inner()
            .stdin
            .extend_from_slice(value.as_ref().as_bytes());
    }
}

impl Read for MockStdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n: usize = 0;
        while !self.stdin.is_empty() && n < buf.len() {
            buf[n] = self.stdin.remove(0);
            n += 1;
        }
        Ok(n)
    }
}

impl MockStdlib {
    pub fn new() -> MockStdlib {
        MockStdlib {
            env: HashMap::new(),
        }
    }
}

impl MockStdout {
    /// Gets the captured output of stdout as-is, without trimming or removing CRLF
    pub fn output_exact(&self) -> String {
        let bytes = self.inner();
        String::from_utf8(bytes.clone()).unwrap()
    }

    /// Gets the captured output of stdout, trimmed and without CRLF
    pub fn output(&self) -> String {
        self.output_exact()
            .trim()
            .trim_matches(|ch| ch == '\r' || ch == '\n')
            .to_string()
    }

    /// Gets the captured output of stdout as a collection of lines.
    /// Each line is trimmed of whitespace and CRLF.
    pub fn output_lines(&self) -> Vec<String> {
        self.output_exact()
            .trim_matches(|ch| ch == '\r' || ch == '\n')
            .split("\r\n")
            .map(|x| x.trim().to_string())
            .collect()
    }

    /// Gets the captures output of stdout as a collection of lines.
    /// Lines are not trimmed.
    pub fn output_lines_exact(&self) -> Vec<String> {
        self.output_exact()
            .split("\r\n")
            .map(|x| x.to_string())
            .collect()
    }
}

impl Stdlib for MockStdlib {
    fn system(&self) {
        println!("would have exited")
    }

    fn get_env_var(&self, name: &String) -> String {
        match self.env.get(name) {
            Some(x) => x.clone(),
            None => String::new(),
        }
    }

    fn set_env_var(&mut self, name: String, value: String) {
        self.env.insert(name, value);
    }
}

impl MockInterpreter {
    pub fn get_variable_str(&self, name: &str) -> Variant {
        let dim_name = DimName::parse(name);
        self.context().get_r_value(&dim_name).unwrap().clone()
    }
}

#[macro_export]
macro_rules! assert_has_variable {
    ($int:expr, $name:expr, $expected_value:expr) => {
        assert_eq!(
            $int.get_variable_str($name),
            crate::variant::Variant::from($expected_value)
        );
    };
}

#[macro_export]
macro_rules! assert_err {
    ($program:expr, $expected_err:expr, $expected_row:expr, $expected_col:expr) => {
        assert_eq!(
            crate::interpreter::test_utils::interpret_err($program),
            crate::common::ErrorEnvelope::Pos(
                $expected_err,
                crate::common::Location::new($expected_row, $expected_col)
            )
        );
    };
}

#[macro_export]
macro_rules! assert_prints_nothing {
    ($program:expr) => {
        let mut interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdout().output_exact(), "");
    };
}

#[macro_export]
macro_rules! assert_prints {
    ($program:expr, $($x:expr),+) => (
        let mut interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdout().output_lines(), vec![$($x),+]);
    );
}

#[macro_export]
macro_rules! assert_prints_exact {
    ($program:expr, $($x:expr),+) => (
        let mut interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdout().output_lines_exact(), vec![$($x),+]);
    );
}

#[macro_export]
macro_rules! assert_lprints_exact {
    ($program:expr, $($x:expr),+) => (
        let mut interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.lpt1().output_lines_exact(), vec![$($x),+]);
    );
}
