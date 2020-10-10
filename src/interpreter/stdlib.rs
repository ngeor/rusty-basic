use crate::interpreter::input_source::{InputSource, ReadInputSource};
use crate::interpreter::interpreter::Lpt1Write;
use crate::interpreter::printer::{Printer, WritePrinter};
use std::io::{Stdin, Stdout, Write};

// TODO trait Reader like Printer read_until_comma read_until_eol

/// The standard functions that QBasic offers
pub trait Stdlib: InputSource + Printer {
    type LPT1: Write;

    /// Implementation of SYSTEM
    fn system(&self);

    /// Gets an environment variable (used by built-in function ENVIRON$)
    fn get_env_var(&self, name: &String) -> String;

    /// Sets an environment variable (used by built-in sub ENVIRON)
    fn set_env_var(&mut self, name: String, value: String);

    fn lpt1(&mut self) -> &mut WritePrinter<Self::LPT1>;

    // TODO stdin stdout
}

// TODO DefaultStdlib<W: Write, R: Read>
pub struct DefaultStdlib {
    stdin: ReadInputSource<Stdin>,
    stdout: WritePrinter<Stdout>,
    lpt1: WritePrinter<Lpt1Write>,
}

impl DefaultStdlib {
    pub fn new() -> Self {
        Self {
            stdin: ReadInputSource::new(std::io::stdin()),
            stdout: WritePrinter::new(std::io::stdout()),
            lpt1: WritePrinter::new(Lpt1Write {}),
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

impl InputSource for DefaultStdlib {
    fn eof(&mut self) -> std::io::Result<bool> {
        self.stdin.eof()
    }

    fn input(&mut self) -> std::io::Result<String> {
        self.stdin.input()
    }

    fn line_input(&mut self) -> std::io::Result<String> {
        self.stdin.line_input()
    }
}

impl Stdlib for DefaultStdlib {
    type LPT1 = Lpt1Write;

    fn system(&self) {
        std::process::exit(0)
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

    fn lpt1(&mut self) -> &mut WritePrinter<Self::LPT1> {
        &mut self.lpt1
    }
}
