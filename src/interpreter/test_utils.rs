use crate::common::*;
use crate::instruction_generator;
use crate::instruction_generator::InstructionNode;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter;
use crate::linter::{ResolvedDeclaredName, ResolvedTypeDefinition};
use crate::parser::{parse_main_file, parse_main_str, QualifiedName};
use crate::variant::Variant;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

pub fn generate_instructions<T>(input: T) -> Vec<InstructionNode>
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).unwrap();
    let linted_program = linter::lint(program).unwrap();
    instruction_generator::generate_instructions(linted_program)
}

pub fn interpret<T>(input: T) -> Interpreter<MockStdlib>
where
    T: AsRef<[u8]> + 'static,
{
    let instructions = generate_instructions(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = Interpreter::new(MockStdlib::new());
    interpreter
        .interpret(instructions)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_with_stdlib<T, TStdlib>(input: T, stdlib: TStdlib) -> Interpreter<TStdlib>
where
    T: AsRef<[u8]> + 'static,
    TStdlib: Stdlib,
{
    let instructions = generate_instructions(input);
    let mut interpreter = Interpreter::new(stdlib);
    interpreter
        .interpret(instructions)
        .map(|_| interpreter)
        .unwrap()
}

pub fn interpret_err<T>(input: T) -> QErrorNode
where
    T: AsRef<[u8]> + 'static,
{
    let instructions = generate_instructions(input);
    let mut interpreter = Interpreter::new(MockStdlib::new());
    interpreter.interpret(instructions).unwrap_err()
}

pub fn interpret_file<S, TStdlib>(
    filename: S,
    stdlib: TStdlib,
) -> Result<Interpreter<TStdlib>, QErrorNode>
where
    S: AsRef<str>,
    TStdlib: Stdlib,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    let program = parse_main_file(f).unwrap();
    let linted_program = linter::lint(program).unwrap();
    let instructions = instruction_generator::generate_instructions(linted_program);
    let mut interpreter = Interpreter::new(stdlib);
    interpreter.interpret(instructions).map(|_| interpreter)
}

#[derive(Debug)]
pub struct MockStdlib {
    next_input: Vec<String>,
    pub output: Vec<String>,
    pub env: HashMap<String, String>,
}

impl MockStdlib {
    pub fn new() -> MockStdlib {
        MockStdlib {
            next_input: vec![],
            output: vec![],
            env: HashMap::new(),
        }
    }

    pub fn add_next_input<S: AsRef<str>>(&mut self, value: S) {
        self.next_input.push(value.as_ref().to_string())
    }
}

impl Stdlib for MockStdlib {
    fn print(&mut self, args: Vec<String>) {
        let mut is_first = true;
        let mut buf = String::new();
        for arg in args {
            if is_first {
                is_first = false;
            } else {
                buf.push(' ');
            }
            buf.push_str(&arg);
        }

        println!("{}", buf);
        self.output.push(buf);
    }

    fn system(&self) {
        println!("would have exited")
    }

    fn input(&mut self) -> std::io::Result<String> {
        Ok(self.next_input.remove(0))
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

impl<S: Stdlib> Interpreter<S> {
    pub fn get_variable_str(&self, name: &str) -> Variant {
        let QualifiedName { name, qualifier } = QualifiedName::try_from(name).unwrap();
        let resolved_declared_name = ResolvedDeclaredName {
            name,
            type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
        };
        self.context_ref()
            .get_r_value(&resolved_declared_name)
            .unwrap()
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
    ($program:expr, $expected_msg:expr, $expected_row:expr, $expected_col:expr) => {
        // for backwards compatibility with older tests
        let expected_interpreter_err = if $expected_msg == "Overflow" {
            crate::common::QError::Overflow
        } else {
            crate::common::QError::Other(format!("{}", $expected_msg))
        };
        assert_eq!(
            crate::interpreter::test_utils::interpret_err($program),
            crate::common::ErrorEnvelope::Pos(
                expected_interpreter_err,
                crate::common::Location::new($expected_row, $expected_col)
            )
        );
    };
}

#[macro_export]
macro_rules! assert_prints {
    ($program:expr; nothing) => {
        let interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdlib.output, Vec::<String>::new());
    };
    ($program:expr, $($x:expr),+) => (
        let interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdlib.output, vec![$($x),+]);
    );
    //($program:expr, $($x:expr,)*) => ($crate::assert_prints![$program, $($x),*])

}
