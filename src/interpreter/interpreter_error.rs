use crate::common::Location;

/// A collection of coordinates in the program where an error occurred
pub type Stacktrace = Vec<Location>;

/// The error type of the interpreter
#[derive(Debug, PartialEq)]
pub struct InterpreterError {
    message: String,
    stacktrace: Stacktrace,
}

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

    pub fn merge_pos(self, pos: Location) -> InterpreterError {
        let mut new_vec = self.stacktrace;
        new_vec.push(pos);
        InterpreterError::new(self.message, new_vec)
    }

    #[cfg(test)]
    pub fn message(&self) -> &String {
        &self.message
    }
}
