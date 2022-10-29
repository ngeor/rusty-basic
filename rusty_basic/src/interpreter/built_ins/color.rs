use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::variant::QBNumberCast;
use rusty_common::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let flags: i32 = interpreter.context()[0].try_cast()?;
    let is_foreground_present = flags & 0x01 != 0;
    let is_background_present = flags & 0x02 != 0;
    if is_foreground_present {
        let foreground_color: i32 = interpreter.context()[1].try_cast()?;
        if is_background_present {
            // set both
            let background_color: i32 = interpreter.context()[2].try_cast()?;
            interpreter.screen().foreground_color(foreground_color)?;
            interpreter.screen().background_color(background_color)
        } else {
            // only set foreground color
            interpreter.screen().foreground_color(foreground_color)
        }
    } else if is_background_present {
        // only set background color
        let background_color: i32 = interpreter.context()[1].try_cast()?;
        interpreter.screen().background_color(background_color)
    } else {
        // should not happen
        Ok(())
    }
}
