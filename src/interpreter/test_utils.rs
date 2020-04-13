use crate::common::Location;
use crate::interpreter::variant::Variant;
use crate::interpreter::{Interpreter, InterpreterError, Result, Stdlib, VariableGetter};
use crate::parser::{Name, NameNode, Parser};
use std::fs::File;

pub fn interpret<T>(input: T) -> Interpreter<MockStdlib>
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    let program = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(MockStdlib::new());
    interpreter.interpret(program).map(|_| interpreter).unwrap()
}

pub fn interpret_with_stdlib<T, TStdlib>(input: T, stdlib: TStdlib) -> Interpreter<TStdlib>
where
    T: AsRef<[u8]>,
    TStdlib: Stdlib,
{
    let mut parser = Parser::from(input);
    let program = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(stdlib);
    interpreter.interpret(program).map(|_| interpreter).unwrap()
}

pub fn interpret_err<T>(input: T) -> InterpreterError
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    let program = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(MockStdlib::new());
    interpreter.interpret(program).unwrap_err()
}

pub fn interpret_file<S, TStdlib>(filename: S, stdlib: TStdlib) -> Result<Interpreter<TStdlib>>
where
    S: AsRef<str>,
    TStdlib: Stdlib,
{
    let file_path = format!("fixtures/{}", filename.as_ref());
    let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
    let program = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(stdlib);
    interpreter.interpret(program).map(|_| interpreter)
}

#[derive(Debug)]
pub struct MockStdlib {
    next_input: Vec<String>,
    pub output: Vec<String>,
}

impl MockStdlib {
    pub fn new() -> MockStdlib {
        MockStdlib {
            next_input: vec![],
            output: vec![],
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
}

#[macro_export]
macro_rules! assert_has_variable {
    ($int:expr, $name:expr, $expected_value:expr) => {
        assert_eq!(
            $int.get_variable($name).unwrap(),
            &Variant::from($expected_value)
        );
    };
}

pub struct AssignmentBuilder {
    variable_name: NameNode,
    program: String,
}

impl AssignmentBuilder {
    pub fn new(variable_literal: &str) -> AssignmentBuilder {
        AssignmentBuilder {
            variable_name: NameNode::new(Name::from(variable_literal), Location::new(1, 1)),
            program: String::new(),
        }
    }

    pub fn literal(&mut self, expression_literal: &str) -> &mut Self {
        if self.program.is_empty() {
            self.program = format!("{} = {}", self.variable_name.element(), expression_literal);
            self
        } else {
            panic!("Cannot re-assign program")
        }
    }

    pub fn assert_eq<T>(&self, expected_value: T)
    where
        Variant: From<T>,
    {
        if self.program.is_empty() {
            panic!("Program was not set")
        } else {
            let interpreter = interpret(&self.program);
            assert_eq!(
                interpreter.get_variable(&self.variable_name).unwrap(),
                &Variant::from(expected_value)
            );
        }
    }

    pub fn assert_err(&self) {
        if self.program.is_empty() {
            panic!("Program was not set");
        } else {
            assert_eq!(
                interpret_err(&self.program),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 1))
            );
        }
    }
}

pub fn assert_assign(variable_literal: &str) -> AssignmentBuilder {
    AssignmentBuilder::new(variable_literal)
}

pub fn assert_input<T>(raw_input: &str, variable_name: &str, expected_value: T)
where
    Variant: From<T>,
{
    let mut stdlib = MockStdlib::new();
    stdlib.add_next_input(raw_input);
    let input = format!("INPUT {}", variable_name);
    let interpreter = interpret_with_stdlib(input, stdlib);
    assert_has_variable!(interpreter, variable_name, expected_value);
}
