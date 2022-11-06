use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;

#[cfg(windows)]
pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), RuntimeError> {
    windows_impl::beep();
    Ok(())
}

#[cfg(not(windows))]
pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), RuntimeError> {
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
