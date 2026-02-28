use crate::RuntimeError;

#[cfg(windows)]
pub unsafe fn get_indicator_keys() -> Result<u8, RuntimeError> {
    windows_impl::get_indicator_keys()
}

#[cfg(windows)]
pub unsafe fn set_indicator_keys(flags: u8) -> Result<(), RuntimeError> {
    windows_impl::set_indicator_keys(flags)
}

#[cfg(not(windows))]
pub unsafe fn get_indicator_keys() -> Result<u8, RuntimeError> {
    // TODO implement get_indicator_keys for other platforms
    Ok(0)
}

#[cfg(not(windows))]
pub unsafe fn set_indicator_keys(_flags: u8) -> Result<(), RuntimeError> {
    // TODO implement set_indicator_keys for other platforms
    Ok(())
}

#[cfg(windows)]
mod windows_impl {
    extern crate winapi;

    use winapi::um::winuser::{
        GetKeyboardState, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, SendInput, VK_CAPITAL,
        VK_NUMLOCK, VK_SCROLL,
    };

    use crate::RuntimeError;

    const KEYS_FLAGS: [(i32, u8); 3] = [(VK_NUMLOCK, 1), (VK_CAPITAL, 2), (VK_SCROLL, 4)];

    pub unsafe fn get_indicator_keys() -> Result<u8, RuntimeError> {
        let keyboard_state = get_keyboard_state()?;
        let mut result: u8 = 0;
        for (key, flag) in &KEYS_FLAGS {
            if keyboard_state[*key as usize] != 0 {
                result |= *flag;
            }
        }
        Ok(result)
    }

    pub unsafe fn set_indicator_keys(flags: u8) -> Result<(), RuntimeError> {
        let keyboard_state = get_keyboard_state()?;
        let mut inputs: Vec<INPUT> = vec![];
        for (key, flag) in &KEYS_FLAGS {
            if should_toggle(&keyboard_state, flags, *key, *flag) {
                inputs.push(new_input(*key, false));
                inputs.push(new_input(*key, true));
            }
        }
        if !inputs.is_empty() {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
        Ok(())
    }

    unsafe fn get_keyboard_state() -> Result<[u8; 256], RuntimeError> {
        let mut buf = [0_u8; 256];
        let result = GetKeyboardState(buf.as_mut_ptr());
        if result != 0 {
            Ok(buf)
        } else {
            Err(RuntimeError::Other(
                "Could not get keyboard state".to_owned(),
            ))
        }
    }

    fn should_toggle(buf: &[u8], flags: u8, key: i32, flags_bit: u8) -> bool {
        let is_buf_on = buf[key as usize] != 0;
        let is_flag_on = flags & flags_bit != 0;
        is_buf_on != is_flag_on
    }

    unsafe fn new_input(key: i32, key_up: bool) -> INPUT {
        let mut result: INPUT = std::mem::zeroed();
        result.type_ = INPUT_KEYBOARD;
        result.u.ki_mut().wVk = key as u16;
        if key_up {
            result.u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
        }
        result
    }
}
