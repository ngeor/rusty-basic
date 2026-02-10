mod arguments;
mod built_ins;
mod byte_size;
mod context;
mod data_segment;
mod default_stdlib;
pub mod error;
mod handlers;
mod indexed_map;
mod interpreter_trait;
mod io;
mod keyboard;
mod lpt1_write;
mod main;
mod print;
mod read_input;
mod registers;
mod screen;
mod stdlib;
mod string_utils;
mod variables;
mod variant_casts;
mod write_printer;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

pub use self::interpreter_trait::InterpreterTrait;
pub use self::main::new_default_interpreter;
pub use self::stdlib::*;

fn is_cr_lf(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}
