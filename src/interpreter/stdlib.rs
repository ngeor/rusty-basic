use crate::interpreter::io::read_until_comma_or_eol;
use crate::variant::Variant;

pub trait Printer {
    /// Implementation of PRINT x[, y, z]
    /// Mutable because of the test implementation
    fn print(&mut self, s: String) -> std::io::Result<usize>;

    fn get_last_print_col(&self) -> usize;

    fn set_last_print_col(&mut self, col: usize);
}

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

pub struct DefaultStdlib {
    last_print_col: usize,
    stdin_buffer: String,
}

impl DefaultStdlib {
    pub fn new() -> Self {
        Self {
            last_print_col: 0,
            stdin_buffer: String::new(),
        }
    }
}

pub enum PrintVal {
    Comma,
    Semicolon,
    NewLine,
    Value(Variant),
}

impl PrintVal {
    pub fn print<T: Printer>(&self, stdlib: &mut T) -> std::io::Result<usize> {
        match self {
            Self::Comma => {
                let col = stdlib.get_last_print_col();
                let len = 14 - col % 14;
                let s: String = (0..len).map(|_| ' ').collect();
                stdlib.set_last_print_col(col + len);
                stdlib.print(s)
            }
            Self::Semicolon => Ok(0),
            Self::NewLine => {
                stdlib.set_last_print_col(0);
                stdlib.print("\r\n".to_string())
            }
            Self::Value(v) => {
                // TODO what if s contains new line
                let s = v.to_string();
                stdlib.set_last_print_col(stdlib.get_last_print_col() + s.len());
                stdlib.print(s)
            }
        }
    }
}

impl Printer for DefaultStdlib {
    fn print(&mut self, s: String) -> std::io::Result<usize> {
        print!("{}", s);
        Ok(s.len())
    }

    fn get_last_print_col(&self) -> usize {
        self.last_print_col
    }

    fn set_last_print_col(&mut self, col: usize) {
        self.last_print_col = col;
    }
}

impl Stdlib for DefaultStdlib {
    fn system(&self) {
        std::process::exit(0)
    }

    fn input(&mut self) -> std::io::Result<String> {
        let (_, remainder, line) =
            read_until_comma_or_eol(std::io::stdin().lock(), self.stdin_buffer.clone())?;
        self.stdin_buffer = remainder;
        Ok(line)
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
