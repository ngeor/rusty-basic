use super::*;
use crate::common::{Location, StripLocation};
use crate::parser::Parser;
use std::fs::File;

pub fn interpret<T, TStdlib>(input: T, stdlib: TStdlib) -> Result<Interpreter<TStdlib>>
where
    T: AsRef<[u8]>,
    TStdlib: Stdlib,
{
    let mut parser = Parser::from(input);
    let program = parser.parse().unwrap();
    let mut interpreter = Interpreter::new(stdlib);
    interpreter.interpret(program).map(|_| interpreter)
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

pub trait InterpreterAssertions {
    fn has_variable<TVar>(&self, variable_name: &str, expected_value: TVar)
    where
        Variant: From<TVar>;

    fn has_variable_close_enough(&self, variable_name: &str, expected_value: f64);
}

const EPSILON_SINGLE: f32 = 0.000001;
const EPSILON_DOUBLE: f64 = 0.000001;

impl<S: Stdlib> InterpreterAssertions for Interpreter<S> {
    fn has_variable<TVar>(&self, variable_name: &str, expected_value: TVar)
    where
        Variant: From<TVar>,
    {
        assert_eq!(
            self.get_variable(variable_name).unwrap(),
            &Variant::from(expected_value)
        );
    }

    fn has_variable_close_enough(&self, variable_name: &str, expected_value: f64) {
        match self.get_variable(variable_name).unwrap() {
            Variant::VDouble(actual_value) => {
                assert!((expected_value - actual_value).abs() <= EPSILON_DOUBLE);
            }
            _ => panic!("Expected double variable"),
        }
    }
}

pub fn assert_close_enough(actual: Variant, expected: Variant) {
    match actual {
        Variant::VSingle(a) => match expected {
            Variant::VSingle(e) => {
                assert!((e - a).abs() <= EPSILON_SINGLE);
            }
            _ => panic!("Type mismatch"),
        },
        Variant::VDouble(a) => match expected {
            Variant::VDouble(e) => {
                assert!((e - a).abs() <= EPSILON_DOUBLE);
            }
            _ => panic!("Type mismatch"),
        },
        _ => unimplemented!(),
    }
}

pub struct AssignmentBuilder {
    variable_name: NameNode,
    program: String,
}

impl AssignmentBuilder {
    pub fn new(variable_literal: &str) -> AssignmentBuilder {
        AssignmentBuilder {
            variable_name: NameNode::from(variable_literal),
            program: String::new(),
        }
    }

    pub fn literal(&mut self, expression_literal: &str) -> &mut Self {
        if self.program.is_empty() {
            self.program = format!(
                "{} = {}",
                self.variable_name.clone().strip_location(),
                expression_literal
            );
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
            let interpreter = interpret(&self.program, MockStdlib::new()).unwrap();
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
                interpret(&self.program, MockStdlib::new()).unwrap_err(),
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
    let interpreter = interpret(input, stdlib).unwrap();
    interpreter.has_variable(variable_name, expected_value);
}
