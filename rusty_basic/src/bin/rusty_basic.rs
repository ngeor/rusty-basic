use std::env;
use std::fs::File;

use rusty_basic::instruction_generator::{generate_instructions, unwrap_linter_context};
use rusty_basic::interpreter::{InterpreterTrait, new_default_interpreter};
use rusty_linter::{Context, lint};
use rusty_parser::{Program, parse_main_file};

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
    match parse_main_file(f) {
        Ok(program) => on_parsed(program, run_options),
        Err(e) => eprintln!("Could not parse program. {:?}", e),
    }
}

fn on_parsed(program: Program, run_options: RunOptions) {
    match lint(program) {
        Ok((linted_program, linter_context)) => {
            on_linted(linted_program, linter_context, run_options)
        }
        Err(e) => eprintln!("Could not lint program. {:?}", e),
    }
}

fn on_linted(program: Program, linter_context: Context, run_options: RunOptions) {
    let (linter_names, user_defined_types) = unwrap_linter_context(linter_context);
    let instruction_generator_result = generate_instructions(program, linter_names);
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
    env::args().nth(1).unwrap_or_default()
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
            .unwrap_or_else(|_| panic!("Could not find program {}", &self.filename))
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
