use crate::common::*;
use crate::instruction_generator::test_utils::generate_instructions_str_with_types;
use crate::instruction_generator::{generate_instructions, InstructionGeneratorResult};
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::screen::{CrossTermScreen, HeadlessScreen};
use crate::interpreter::write_printer::WritePrinter;
use crate::interpreter::Stdlib;
use crate::linter;
use crate::linter::HasUserDefinedTypes;
use crate::parser::{parse_main_file, Name};
use crate::variant::Variant;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

type MockStdout = WritePrinter<Vec<u8>>;

pub trait MockInterpreterTrait:
    InterpreterTrait<TStdOut = MockStdout, TStdIn = ReadInputSource<MockStdin>, TLpt1 = MockStdout>
{
    // TODO #[deprecated]
    fn get_variable_str(&self, name: &str) -> Variant {
        let name = Name::from(name);
        self.context().get_by_name(&name)
    }
}

impl<S> MockInterpreterTrait for S where
    S: InterpreterTrait<
        TStdOut = MockStdout,
        TStdIn = ReadInputSource<MockStdin>,
        TLpt1 = MockStdout,
    >
{
}

fn mock_interpreter_for_user_defined_types<U: HasUserDefinedTypes>(
    user_defined_types_holder: U,
) -> impl MockInterpreterTrait {
    let stdlib = MockStdlib::new();
    mock_interpreter_for_user_defined_types_stdlib(user_defined_types_holder, stdlib)
}

fn mock_interpreter_for_user_defined_types_stdlib<U: HasUserDefinedTypes>(
    user_defined_types_holder: U,
    stdlib: MockStdlib,
) -> impl MockInterpreterTrait {
    let stdin = ReadInputSource::new(MockStdin { stdin: vec![] });
    let stdout = WritePrinter::new(vec![]);
    let lpt1 = WritePrinter::new(vec![]);
    if std::env::var("USE_REAL_SCREEN")
        .unwrap_or_default()
        .is_empty()
    {
        Interpreter::new(
            stdlib,
            stdin,
            stdout,
            lpt1,
            HeadlessScreen {},
            user_defined_types_holder,
        )
    } else {
        Interpreter::new(
            stdlib,
            stdin,
            stdout,
            lpt1,
            CrossTermScreen::new(),
            user_defined_types_holder,
        )
    }
}

pub fn mock_interpreter_for_input<T>(
    input: T,
) -> (InstructionGeneratorResult, impl MockInterpreterTrait)
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_generator_result, user_defined_types_holder) =
        generate_instructions_str_with_types(input);
    // println!("{:#?}", instruction_generator_result.instructions);
    (
        instruction_generator_result,
        mock_interpreter_for_user_defined_types(user_defined_types_holder),
    )
}

pub fn interpret<T>(input: T) -> impl MockInterpreterTrait
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_generator_result, mut interpreter) = mock_interpreter_for_input(input);
    interpreter
        .interpret(instruction_generator_result)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_err<T>(input: T) -> QErrorNode
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_generator_result, mut interpreter) = mock_interpreter_for_input(input);
    interpreter
        .interpret(instruction_generator_result)
        .unwrap_err()
}

pub fn interpret_with_raw_input<T>(input: T, raw_input: &str) -> impl MockInterpreterTrait
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_generator_result, user_defined_types_holder) =
        generate_instructions_str_with_types(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = mock_interpreter_for_user_defined_types(user_defined_types_holder);
    if !raw_input.is_empty() {
        interpreter.stdin().add_next_input(raw_input);
    }
    interpreter
        .interpret(instruction_generator_result)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_with_env<T>(input: T, stdlib: MockStdlib) -> impl MockInterpreterTrait
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_generator_result, user_defined_types_holder) =
        generate_instructions_str_with_types(input);
    // println!("{:#?}", instructions);
    let mut interpreter =
        mock_interpreter_for_user_defined_types_stdlib(user_defined_types_holder, stdlib);
    interpreter
        .interpret(instruction_generator_result)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_file<S>(filename: S) -> Result<impl MockInterpreterTrait, QErrorNode>
where
    S: AsRef<str>,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    let program = parse_main_file(f).unwrap();
    let (linted_program, user_defined_types) = linter::lint(program).unwrap();
    let instruction_generator_result = generate_instructions(linted_program);
    let mut interpreter = mock_interpreter_for_user_defined_types(user_defined_types);
    interpreter
        .interpret(instruction_generator_result)
        .map(|_| interpreter)
}

pub fn interpret_file_with_raw_input<S>(
    filename: S,
    raw_input: &str,
) -> Result<impl MockInterpreterTrait, QErrorNode>
where
    S: AsRef<str>,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    let program = parse_main_file(f).unwrap();
    let (linted_program, user_defined_types) = linter::lint(program).unwrap();
    let instruction_generator_result = generate_instructions(linted_program);
    let mut interpreter = mock_interpreter_for_user_defined_types(user_defined_types);
    if !raw_input.is_empty() {
        interpreter.stdin().add_next_input(raw_input);
    }
    interpreter
        .interpret(instruction_generator_result)
        .map(|_| interpreter)
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
        let bytes: Vec<u8> = self.inner();
        String::from_utf8(bytes).unwrap()
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

    fn get_env_var(&self, name: &str) -> String {
        match self.env.get(name) {
            Some(x) => x.clone(),
            None => String::new(),
        }
    }

    fn set_env_var(&mut self, name: String, value: String) {
        self.env.insert(name, value);
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
macro_rules! assert_interpreter_err {
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
