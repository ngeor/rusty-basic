mod common;
mod interpreter;
mod lexer;
mod parser;
mod reader;
use std::env;
use std::fs::File;

use interpreter::{DefaultStdlib, Interpreter};
use parser::Parser;

fn main() {
    let filename = env::args()
        .skip(1)
        .take(1)
        .last()
        .expect("The first argument should be the program to run");
    let f = File::open(&filename).expect(format!("Could not find program {}", filename).as_ref());
    let mut parser = Parser::from(f);
    let mut interpreter = Interpreter::new(DefaultStdlib {});
    interpreter.interpret(parser.parse().unwrap()).unwrap();
}
