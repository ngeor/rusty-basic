use crate::common::Location;

/// A collection of coordinates in the program where an error occurred
pub type Stacktrace = Vec<Location>;

/// The error type of the interpreter
#[derive(Debug, PartialEq)]
pub struct InterpreterError {
    message: String,
    stacktrace: Stacktrace,
}

pub type Result<T> = std::result::Result<T, InterpreterError>;

impl InterpreterError {
    pub fn new<S: AsRef<str>>(msg: S, stacktrace: Stacktrace) -> InterpreterError {
        InterpreterError {
            message: msg.as_ref().to_string(),
            stacktrace,
        }
    }

    pub fn new_with_pos<S: AsRef<str>>(msg: S, pos: Location) -> InterpreterError {
        InterpreterError::new(msg, vec![pos])
    }

    pub fn with_existing_stacktrace(self, stacktrace: &Stacktrace) -> InterpreterError {
        let mut new_vec = vec![];
        for x in self.stacktrace {
            new_vec.push(x);
        }
        for x in stacktrace.iter() {
            new_vec.push(*x);
        }
        InterpreterError::new(self.message, new_vec)
    }
}

pub fn err<T, S: AsRef<str>>(msg: S, pos: Location) -> Result<T> {
    Err(InterpreterError::new_with_pos(msg, pos))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    #[test]
    fn on_error_go_to_label() {
        let input = r#"
        ON ERROR GOTO ErrTrap
        Environ "ShouldHaveAnEqualsSignInHereSomewhere"
        PRINT "Will not print this"
        SYSTEM
        ErrTrap:
            PRINT "Saved by the bell"
        "#;
        let interpreter = interpret(input);
        assert_eq!(interpreter.stdlib.output, vec!["Saved by the bell"]);
    }

    #[test]
    fn on_error_go_to_missing_label() {
        let input = r#"
        ON ERROR GOTO ErrTrap
        "#;
        assert_linter_err!(input, LinterError::LabelNotFound, 2, 9);
    }
}
