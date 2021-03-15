mod arguments;
mod built_ins;
pub mod context;
mod default_stdlib;
mod input;
pub mod interpreter;
pub mod interpreter_trait;
mod io;
mod lpt1_write;
mod print;
mod printer;
mod read_input;
mod registers;
mod stdlib;
mod variables;
mod write_printer;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;
