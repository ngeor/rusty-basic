use crate::common::*;
use crate::variant::VariantError;

#[derive(Clone, Debug, PartialEq)]
pub enum InterpreterError {
    ForLoopZeroStep,
    TypeMismatch,
    Overflow,
    FileNotFound,
    IllegalFunctionCall,
    DivisionByZero,
    IO(String),
    Other(String),
}

pub type InterpreterErrorNode = ErrorEnvelope<InterpreterError>;

impl From<std::io::Error> for InterpreterError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::FileNotFound
        } else {
            Self::IO(e.to_string())
        }
    }
}

impl From<&str> for InterpreterError {
    fn from(x: &str) -> Self {
        x.to_string().into()
    }
}

impl From<String> for InterpreterError {
    fn from(x: String) -> Self {
        Self::Other(x)
    }
}

impl From<VariantError> for InterpreterError {
    fn from(x: VariantError) -> Self {
        match x {
            VariantError::Overflow => Self::Overflow,
            VariantError::DivisionByZero => Self::DivisionByZero,
            VariantError::TypeMismatch => Self::TypeMismatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

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
}
