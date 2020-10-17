/// The standard functions that QBasic offers
pub trait Stdlib {
    /// Implementation of SYSTEM
    fn system(&self);

    /// Gets an environment variable (used by built-in function ENVIRON$)
    fn get_env_var(&self, name: &String) -> String;

    /// Sets an environment variable (used by built-in sub ENVIRON)
    fn set_env_var(&mut self, name: String, value: String);
}

pub struct DefaultStdlib {}

impl DefaultStdlib {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stdlib for DefaultStdlib {
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
}
