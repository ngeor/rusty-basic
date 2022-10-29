use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[cfg(windows)]
pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QError> {
    windows_impl::beep();
    Ok(())
}

#[cfg(not(windows))]
pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QError> {
    Ok(())
}

#[cfg(windows)]
mod windows_impl {
    extern crate winapi;

    use winapi::um::winuser::MessageBeep;

    pub fn beep() {
        unsafe {
            MessageBeep(0);
        }
    }
}
