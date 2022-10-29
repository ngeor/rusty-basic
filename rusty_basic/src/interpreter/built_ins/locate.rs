use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::utils::VariantCasts;
use rusty_common::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let mut iterator = interpreter.context().variables().iter();
    let flags: usize = iterator.next().unwrap().to_positive_int()?;
    let is_row_present = flags & 0x01 != 0;
    let is_col_present = flags & 0x02 != 0;
    let is_cursor_present = flags & 0x04 != 0;
    let row: Option<usize> = if is_row_present {
        Some(iterator.next().unwrap().to_positive_int()?)
    } else {
        None
    };
    let col: Option<usize> = if is_col_present {
        Some(iterator.next().unwrap().to_positive_int()?)
    } else {
        None
    };
    let cursor: Option<usize> = if is_cursor_present {
        Some(iterator.next().unwrap().to_non_negative_int()?)
    } else {
        None
    };
    move_to(interpreter, row, col)?;
    show_hide_cursor(interpreter, cursor)
}

fn move_to<S: InterpreterTrait>(
    interpreter: &S,
    row: Option<usize>,
    col: Option<usize>,
) -> Result<(), QError> {
    if let Some(row) = row {
        if let Some(col) = col {
            interpreter
                .screen()
                .move_to((row - 1) as u16, (col - 1) as u16)
        } else {
            interpreter.screen().move_to((row - 1) as u16, 0)
        }
    } else if col.is_some() {
        // cannot move to a col because the current row is unknown
        Err(QError::IllegalFunctionCall)
    } else {
        Ok(())
    }
}

fn show_hide_cursor<S: InterpreterTrait>(
    interpreter: &S,
    cursor: Option<usize>,
) -> Result<(), QError> {
    match cursor {
        Some(1) => interpreter.screen().show_cursor(),
        Some(0) => interpreter.screen().hide_cursor(),
        Some(_) => Err(QError::IllegalFunctionCall),
        None => Ok(()),
    }
}
