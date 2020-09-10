use std::env;
use std::fs::File;

use rusty_basic::instruction_generator;
use rusty_basic::interpreter::{DefaultStdlib, Interpreter};
use rusty_basic::linter;
use rusty_basic::parser;

// TODO only use the apache hacks if called with a flag or so
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
    let f = File::open(&filename).expect(format!("Could not find program {}", filename).as_ref());
    match parser::parse_main_file(f) {
        Ok(program) => match linter::lint(program) {
            Ok((linted_program, user_defined_types)) => {
                let instructions = instruction_generator::generate_instructions(linted_program);
                set_current_dir(&filename); // Note: only needed to make it work inside Apache.
                let mut interpreter = Interpreter::new(DefaultStdlib {}, user_defined_types);
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
