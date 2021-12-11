use std::env;
use std::fs::File;

use rusty_basic::instruction_generator;
use rusty_basic::interpreter::interpreter::new_default_interpreter;
use rusty_basic::linter;
use rusty_basic::parser;
use rusty_basic::parser::{ProgramNode, UserDefinedTypes};

fn main() {
    let is_running_in_apache = is_running_in_apache();
    let filename = get_filename(is_running_in_apache);
    if filename.is_empty() {
        eprintln!("Please specify the program to run.");
        return;
    }
    let run_options = RunOptions {
        is_running_in_apache,
        filename,
    };
    let f = run_options.open_file();
    match parser::parse_main_file(f) {
        Ok(program) => on_parsed(program, run_options),
        Err(e) => eprintln!("Could not parse program. {:?}", e),
    }
}

fn on_parsed(program: ProgramNode, run_options: RunOptions) {
    match linter::lint(program) {
        Ok((linted_program, user_defined_types)) => {
            on_linted(linted_program, user_defined_types, run_options)
        }
        Err(e) => eprintln!("Could not lint program. {:?}", e),
    }
}

fn on_linted(program: ProgramNode, user_defined_types: UserDefinedTypes, run_options: RunOptions) {
    let instruction_generator_result = instruction_generator::generate_instructions(program);
    let mut interpreter = new_default_interpreter(user_defined_types);
    run_options.set_current_dir_if_apache();
    match interpreter.interpret(instruction_generator_result) {
        Ok(_) => (),
        Err(e) => eprintln!("Runtime error. {:?}", e),
    }
}

fn get_filename(is_running_in_apache: bool) -> String {
    // Normally it should just be the first command line argument.
    // We also check the variable PATH_TRANSLATED in order to make it work inside Apache.
    if is_running_in_apache {
        get_filename_from_env_var()
    } else {
        get_filename_from_args()
    }
}

fn get_filename_from_args() -> String {
    env::args().skip(1).take(1).last().unwrap_or_default()
}

fn get_filename_from_env_var() -> String {
    std::env::var("PATH_TRANSLATED")
        .expect("The PATH_TRANSLATED env variable should be the program to run")
}

/// Checks if we're running inside Apache with mod_cgi.
fn is_running_in_apache() -> bool {
    match std::env::var("SERVER_NAME") {
        Ok(x) => !x.is_empty(),
        Err(_) => false,
    }
}

struct RunOptions {
    filename: String,
    is_running_in_apache: bool,
}

impl RunOptions {
    pub fn open_file(&self) -> File {
        File::open(&self.filename)
            .expect(format!("Could not find program {}", &self.filename).as_ref())
    }

    pub fn set_current_dir_if_apache(&self) {
        if !self.is_running_in_apache {
            return;
        }

        let canonical = std::fs::canonicalize(&self.filename).unwrap();
        let parent = canonical.parent().unwrap();
        std::env::set_current_dir(parent).expect("Could not set current directory");
    }
}
