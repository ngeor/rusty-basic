use crate::common::Result;

/// The standard functions that QBasic offers
pub trait Stdlib {
    /// Implementation of PRINT x[, y, z]
    fn print(&self, args: Vec<String>);

    /// Implementation of SYSTEM
    fn system(&self);

    /// Implementation of INPUT
    fn input(&self) -> Result<String>;
}

pub struct DefaultStdlib {}

impl Stdlib for DefaultStdlib {
    fn print(&self, args: Vec<String>) {
        for a in args {
            print!("{}", a)
        }

        println!("")
    }

    fn system(&self) {
        std::process::exit(0)
    }

    fn input(&self) -> Result<String> {
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => Ok(line.trim_end().to_string()),
            Err(x) => Err(format!("Could not read line: {}", x)),
        }
    }
}
