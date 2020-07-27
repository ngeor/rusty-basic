mod built_ins;
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

fn get_filename() -> String {
    // Normally it should just be the first command line argument.
    // We also check the variable BLR_PROGRAM in order to make it work inside Apache.
    env::args().skip(1).take(1).last().unwrap_or_else(|| {
        std::env::var("REDIRECT_BLR_PROGRAM").unwrap_or_else(|_| {
            std::env::var("BLR_PROGRAM").expect("The first argument should be the program to run")
        })
    })
}

fn set_current_dir(filename: &String) {
    let canonical = std::fs::canonicalize(&filename).unwrap();
    let parent = canonical.parent().unwrap();
    std::env::set_current_dir(parent).expect("Could not set current directory");
}

fn main() {
    let filename = get_filename();
    set_current_dir(&filename); // Note: only needed to make it work inside Apache.
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
