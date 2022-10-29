/// The standard functions that QBasic offers
pub trait Stdlib {
    /// Implementation of SYSTEM
    fn system(&self);

    /// Gets an environment variable (used by built-in function ENVIRON$)
    fn get_env_var(&self, name: &str) -> String;

    /// Sets an environment variable (used by built-in sub ENVIRON)
    fn set_env_var(&mut self, name: String, value: String);
}
