use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use rusty_parser::BuiltInFunction;
use std::time::Duration;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let s: String = poll_one()?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::InKey, s);
    Ok(())
}

fn poll_one() -> Result<String, RuntimeError> {
    if poll(Duration::from_millis(100))? {
        let event = read()?;
        Ok(handle_event(event))
    } else {
        Ok(String::new())
    }
}

fn handle_event(event: Event) -> String {
    if let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event
    {
        handle_key(code, modifiers)
    } else {
        String::new()
    }
}

fn handle_key(code: KeyCode, modifiers: KeyModifiers) -> String {
    match code {
        KeyCode::Char(ch) => {
            if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                String::from(ch)
            } else {
                // TODO
                String::new()
            }
        }
        KeyCode::Enter => String::from(13_u8 as char),
        KeyCode::Tab => {
            if modifiers == KeyModifiers::NONE {
                String::from(9 as char)
            } else if modifiers == KeyModifiers::SHIFT {
                String::from("\0\u{15}")
            } else {
                String::new()
            }
        }
        KeyCode::Up => String::from("\0H"),
        KeyCode::Down => String::from("\0P"),
        KeyCode::Left => String::from("\0K"),
        KeyCode::Right => String::from("\0M"),
        KeyCode::Esc => String::from(27 as char),
        KeyCode::Backspace => String::from(8 as char),
        KeyCode::F(f) => {
            let mut s = String::new();
            s.push(0 as char);
            s.push((58 + f) as char);
            s
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::handle_key;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_mapping_lowercase_letters() {
        for ch in 'a'..='z' {
            assert_eq!(
                handle_key(KeyCode::Char(ch), KeyModifiers::NONE),
                String::from(ch)
            );
        }
    }

    #[test]
    fn test_mapping_uppercase_letters() {
        for ch in 'A'..='Z' {
            assert_eq!(
                handle_key(KeyCode::Char(ch), KeyModifiers::SHIFT),
                String::from(ch)
            );
        }
    }

    #[test]
    fn test_mapping_function_keys() {
        assert_eq!(
            handle_key(KeyCode::F(2), KeyModifiers::NONE),
            String::from("\0<")
        );
        assert_eq!(
            handle_key(KeyCode::F(3), KeyModifiers::NONE),
            String::from("\0=")
        );
    }

    #[test]
    fn test_backspace() {
        assert_eq!(
            handle_key(KeyCode::Backspace, KeyModifiers::NONE),
            String::from(8 as char)
        );
    }

    #[test]
    fn test_enter() {
        assert_eq!(
            handle_key(KeyCode::Enter, KeyModifiers::NONE),
            String::from(13 as char)
        );
    }

    #[test]
    fn test_escape() {
        assert_eq!(
            handle_key(KeyCode::Esc, KeyModifiers::NONE),
            String::from(27 as char)
        );
    }

    #[test]
    fn test_up_arrow() {
        assert_eq!(
            handle_key(KeyCode::Up, KeyModifiers::NONE),
            String::from("\0H")
        );
    }

    #[test]
    fn test_down_arrow() {
        assert_eq!(
            handle_key(KeyCode::Down, KeyModifiers::NONE),
            String::from("\0P")
        );
    }

    #[test]
    fn test_left_arrow() {
        assert_eq!(
            handle_key(KeyCode::Left, KeyModifiers::NONE),
            String::from("\0K")
        );
    }

    #[test]
    fn test_right_arrow() {
        assert_eq!(
            handle_key(KeyCode::Right, KeyModifiers::NONE),
            String::from("\0M")
        );
    }

    #[test]
    fn test_tab() {
        assert_eq!(
            handle_key(KeyCode::Tab, KeyModifiers::NONE),
            String::from(9 as char)
        );
    }

    #[test]
    fn test_shift_tab() {
        assert_eq!(
            handle_key(KeyCode::Tab, KeyModifiers::SHIFT),
            String::from("\0\u{15}")
        );
    }
}
