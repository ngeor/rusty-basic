use crate::interpreter::io::read_until_comma_or_eol;
use crate::interpreter::printer::{Printer, WritePrinter};
use std::io::Stdout;

// TODO trait Reader like Printer read_until_comma read_until_eol

/// The standard functions that QBasic offers
pub trait Stdlib: Printer {
    /// Implementation of SYSTEM
    fn system(&self);

    /// Implementation of INPUT
    /// Mutable because of the test implementation
    fn input(&mut self) -> std::io::Result<String>;

    /// Gets an environment variable (used by built-in function ENVIRON$)
    fn get_env_var(&self, name: &String) -> String;

    /// Sets an environment variable (used by built-in sub ENVIRON)
    fn set_env_var(&mut self, name: String, value: String);
}

// TODO DefaultStdlib<W: Write, R: Read>
pub struct DefaultStdlib {
    stdin_buffer: String,
    stdout: WritePrinter<Stdout>,
}

impl DefaultStdlib {
    pub fn new() -> Self {
        Self {
            stdin_buffer: String::new(),
            stdout: WritePrinter::new(std::io::stdout()),
        }
    }
}

impl Printer for DefaultStdlib {
    fn print(&mut self, s: &str) -> std::io::Result<usize> {
        self.stdout.print(s)
    }

    fn println(&mut self) -> std::io::Result<usize> {
        self.stdout.println()
    }

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize> {
        self.stdout.move_to_next_print_zone()
    }
}

impl Stdlib for DefaultStdlib {
    fn system(&self) {
        std::process::exit(0)
    }

    fn input(&mut self) -> std::io::Result<String> {
        read_until_comma_or_eol(&mut std::io::stdin().lock(), &mut self.stdin_buffer)
    }

    fn get_env_var(&self, name: &String) -> String {
        match std::env::var(name) {
            Ok(x) => x,
            Err(_) => String::new(),
        }
    }

    fn set_env_var(&mut self, name: String, value: String) {
        std::env::set_var(name, value);
    }
}
