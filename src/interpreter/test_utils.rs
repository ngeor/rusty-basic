use crate::common::*;
use crate::instruction_generator::generate_instructions;
use crate::instruction_generator::test_utils::generate_instructions_str_with_types;
use crate::interpreter::{Interpreter, Printer, Stdlib};
use crate::linter;
use crate::linter::DimName;
use crate::parser::parse_main_file;
use crate::variant::Variant;
use std::collections::HashMap;
use std::fs::File;

pub fn interpret<T>(input: T) -> Interpreter<MockStdlib>
where
    T: AsRef<[u8]> + 'static,
{
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    // for i in instructions.iter() {
    //     println!("{:?}", i.as_ref());
    // }
    let mut interpreter = Interpreter::new(MockStdlib::new(), user_defined_types);
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
    let (instructions, user_defined_types) = generate_instructions_str_with_types(input);
    let mut interpreter = Interpreter::new(stdlib, user_defined_types);
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
    let mut interpreter = Interpreter::new(MockStdlib::new(), user_defined_types);
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
    let (linted_program, user_defined_types) = linter::lint(program).unwrap();
    let instructions = generate_instructions(linted_program);
    let mut interpreter = Interpreter::new(stdlib, user_defined_types);
    interpreter.interpret(instructions).map(|_| interpreter)
}

#[derive(Debug)]
pub struct MockStdlib {
    next_input: Vec<String>,
    last_print_col: usize,
    saw_new_line: bool,
    pub output: Vec<String>,
    pub lpt1output: Vec<String>,
    pub env: HashMap<String, String>,
}

impl MockStdlib {
    pub fn new() -> MockStdlib {
        MockStdlib {
            next_input: vec![],
            output: vec![],
            lpt1output: vec![],
            env: HashMap::new(),
            last_print_col: 0,
            saw_new_line: true,
        }
    }

    pub fn add_next_input<S: AsRef<str>>(&mut self, value: S) {
        self.next_input.push(value.as_ref().to_string())
    }
}

impl Printer for MockStdlib {
    fn print(&mut self, s: String) -> std::io::Result<usize> {
        print!("{}", s);
        if self.saw_new_line {
            self.output.push(if s == "\r\n" {
                "".to_string()
            } else {
                s.clone()
            });
        } else if s != "\r\n" {
            self.output.last_mut().unwrap().push_str(s.as_str());
        }
        self.saw_new_line = s == "\r\n";
        Ok(s.len())
    }

    fn get_last_print_col(&self) -> usize {
        self.last_print_col
    }

    fn set_last_print_col(&mut self, col: usize) {
        self.last_print_col = col;
    }
}

impl Stdlib for MockStdlib {
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

#[macro_export]
macro_rules! assert_lprints {
    ($program:expr; nothing) => {
        let interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdlib.lpt1output, Vec::<String>::new());
    };
    ($program:expr, $($x:expr),+) => (
        let interpreter = crate::interpreter::test_utils::interpret($program);
        assert_eq!(interpreter.stdlib.lpt1output, vec![$($x),+]);
    );
    //($program:expr, $($x:expr,)*) => ($crate::assert_prints![$program, $($x),*])

}
