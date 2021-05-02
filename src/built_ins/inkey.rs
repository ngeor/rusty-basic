pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_zero_arguments()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
    use std::time::Duration;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let s: String = poll_one()?;
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::InKey, s);
        Ok(())
    }

    fn poll_one() -> Result<String, QError> {
        if poll(Duration::from_millis(100))? {
            let event = read()?;
            Ok(handle_event(event))
        } else {
            Ok(String::new())
        }
    }

    fn handle_event(event: Event) -> String {
        if let Event::Key(KeyEvent { code, modifiers }) = event {
            handle_key(code, modifiers)
        } else {
            String::new()
        }
    }

    fn handle_key(code: KeyCode, modifiers: KeyModifiers) -> String {
        match code {
            KeyCode::Char(ch) => {
                if modifiers == KeyModifiers::NONE {
                    String::from(ch)
                } else {
                    // TODO
                    String::new()
                }
            }
            KeyCode::Enter => String::from(13_u8 as char),
            KeyCode::Tab => String::from(9 as char),
            KeyCode::Right => String::from("\0M"),
            KeyCode::Down => String::from("\0P"),
            KeyCode::Esc => String::from(27 as char),
            _ => String::new(),
        }
    }
}
