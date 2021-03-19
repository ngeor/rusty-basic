mod arguments;
pub mod context;
pub mod data_segment;
mod default_stdlib;
pub mod interpreter;
pub mod interpreter_trait;
pub mod io;
mod lpt1_write;
mod print;
mod read_input;
mod registers;
pub mod utils;
mod variables;
mod write_printer;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;

/// The standard functions that QBasic offers
pub trait Stdlib {
    /// Implementation of SYSTEM
    fn system(&self);

    /// Gets an environment variable (used by built-in function ENVIRON$)
    fn get_env_var(&self, name: &str) -> String;

    /// Sets an environment variable (used by built-in sub ENVIRON)
    fn set_env_var(&mut self, name: String, value: String);
}
