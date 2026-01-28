use crate::interpreter::Stdlib;

pub struct DefaultStdlib;

impl Stdlib for DefaultStdlib {
    fn system(&self) {
        std::process::exit(0)
    }

    fn get_env_var(&self, name: &str) -> String {
        std::env::var(name).unwrap_or_default()
    }

    fn set_env_var(&mut self, name: String, value: String) {
        unsafe {
            std::env::set_var(name, value);
        }
    }
}
