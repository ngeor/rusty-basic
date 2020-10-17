use crate::interpreter::stdlib::Stdlib;

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
