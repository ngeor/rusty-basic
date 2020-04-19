/// The standard functions that QBasic offers
pub trait Stdlib {
    /// Implementation of PRINT x[, y, z]
    /// Mutable because of the test implementation
    fn print(&mut self, args: Vec<String>);

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

pub struct DefaultStdlib {}

impl Stdlib for DefaultStdlib {
    fn print(&mut self, args: Vec<String>) {
        for a in args {
            print!("{}", a)
        }

        println!("")
    }

    fn system(&self) {
        std::process::exit(0)
    }

    fn input(&mut self) -> std::io::Result<String> {
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => Ok(line.trim_end().to_string()),
            Err(x) => Err(x),
        }
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
