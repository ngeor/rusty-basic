use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Printer;
use rusty_common::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    match interpreter.screen().get_view_print() {
        Some((start_row, end_row)) => {
            // we don't have a better way of doing this
            let spaces: String = [' '; 80].iter().collect();
            for row in start_row..(end_row + 1) {
                interpreter.screen().move_to(row as u16 - 1, 0)?;
                interpreter.stdout().print(spaces.as_str())?;
            }
            Ok(())
        }
        _ => interpreter.screen().cls(),
    }
}
