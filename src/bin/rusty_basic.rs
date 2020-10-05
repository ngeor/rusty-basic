use std::env;
use std::fs::File;

use rusty_basic::instruction_generator;
use rusty_basic::interpreter::{DefaultStdlib, Interpreter};
use rusty_basic::linter;
use rusty_basic::parser;

fn main() {
    let is_running_in_apache = is_running_in_apache();
    let filename = get_filename(is_running_in_apache);
    let f = File::open(&filename).expect(format!("Could not find program {}", filename).as_ref());
    match parser::parse_main_file(f) {
        Ok(program) => match linter::lint(program) {
            Ok((linted_program, user_defined_types)) => {
                let instructions = instruction_generator::generate_instructions(linted_program);
                if is_running_in_apache {
                    set_current_dir(&filename); // Note: only needed to make it work inside Apache.
                }
                let mut interpreter = Interpreter::new(DefaultStdlib::new(), user_defined_types);
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

fn get_filename(is_running_in_apache: bool) -> String {
    // Normally it should just be the first command line argument.
    // We also check the variable BLR_PROGRAM in order to make it work inside Apache.
    if is_running_in_apache {
        get_filename_from_env_var()
    } else {
        get_filename_from_args()
    }
}

fn get_filename_from_args() -> String {
    env::args()
        .skip(1)
        .take(1)
        .last()
        .expect("First argument should be the program to run")
}

fn get_filename_from_env_var() -> String {
    std::env::var("REDIRECT_BLR_PROGRAM").unwrap_or_else(|_| {
        std::env::var("BLR_PROGRAM")
            .expect("The BLR_PROGRAM env variable should be the program to run")
    })
}

fn set_current_dir(filename: &String) {
    let canonical = std::fs::canonicalize(&filename).unwrap();
    let parent = canonical.parent().unwrap();
    std::env::set_current_dir(parent).expect("Could not set current directory");
}

/// Checks if we're running inside Apache with mod_cgi.
fn is_running_in_apache() -> bool {
    match std::env::var("SERVER_NAME") {
        Ok(x) => !x.is_empty(),
        Err(_) => false,
    }
}
