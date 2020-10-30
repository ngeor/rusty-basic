mod arguments;
mod arguments_stack;
mod built_ins;
pub mod context;
mod default_stdlib;
mod input;
mod instruction_handlers;
pub mod interpreter;
mod interpreter_trait;
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
