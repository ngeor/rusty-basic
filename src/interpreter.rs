mod argument;
mod arguments;
mod arguments_stack;
mod built_ins;
pub mod context;
mod interpreter;
mod io;
mod stdlib;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;

pub use self::interpreter::{Interpreter, SetVariable};
pub use self::stdlib::*;
