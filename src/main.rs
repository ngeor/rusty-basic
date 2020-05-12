mod casting;
mod common;
mod instruction_generator;
mod interpreter;
mod lexer;
mod linter;
mod parser;
mod reader;
mod variant;

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
    match parser.parse() {
        Ok(program) => match linter::lint(program) {
            Ok(linted_program) => {
                let mut interpreter = Interpreter::new(DefaultStdlib {});
                let instructions = instruction_generator::generate_instructions(linted_program);
                match interpreter.interpret(instructions) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Runtime error. {:?}", e),
                }
            }
            Err(e) => eprintln!("Could not lint program. {:?}", e),
        },
        Err(e) => eprintln!("Could not parse program. {:?}", e),
    }
}
