use crate::error_envelope::ErrorEnvelope;
use rusty_linter::LintError;
use rusty_variant::{SubscriptOutOfRangeError, VariantError};

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    BadFileMode,
    BadFileNameOrNumber,
    BadRecordLength,
    BadRecordNumber,
    DivisionByZero,
    ElementNotDefined,
    FieldOverflow,
    FileAlreadyOpen,
    FileNotFound,
    ForLoopZeroStep,
    DeviceIOError(String),
    IllegalFunctionCall,
    InputPastEndOfFile,
    LinterError(LintError),
    OutOfData,
    Overflow,
    ReturnWithoutGoSub,
    SubscriptOutOfRange,
    TypeMismatch,
    VariableRequired,
    Other(String),
    ResumeWithoutError,
}

impl RuntimeError {
    pub fn get_code(&self) -> i32 {
        match self {
            Self::ReturnWithoutGoSub => 3,
            Self::IllegalFunctionCall => 5,
            Self::Overflow => 6,
            Self::SubscriptOutOfRange => 9,
            Self::DivisionByZero => 11,
            Self::TypeMismatch => 13,
            Self::ResumeWithoutError => 20,
            Self::BadFileNameOrNumber => 52,
            Self::FileNotFound => 53,
            Self::FileAlreadyOpen => 55,
            Self::InputPastEndOfFile => 62,
            // the following are not QBasic codes
            Self::Other(_) => 257,
            Self::ForLoopZeroStep => 258,
            _ => panic!("not implemented for {:?}", self),
        }
    }
}

pub type RuntimeErrorPos = ErrorEnvelope<RuntimeError>;

impl From<LintError> for RuntimeError {
    fn from(e: LintError) -> Self {
        match e {
            LintError::Overflow => Self::Overflow,
            LintError::TypeMismatch => Self::TypeMismatch,
            LintError::DivisionByZero => Self::DivisionByZero,
            _ => Self::LinterError(e),
        }
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::FileNotFound
        } else if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::InputPastEndOfFile
        } else {
            Self::DeviceIOError(e.to_string())
        }
    }
}

impl From<VariantError> for RuntimeError {
    fn from(e: VariantError) -> Self {
        match e {
            VariantError::DivisionByZero => Self::DivisionByZero,
            VariantError::Overflow => Self::Overflow,
            VariantError::TypeMismatch => Self::TypeMismatch,
        }
    }
}

impl From<SubscriptOutOfRangeError> for RuntimeError {
    fn from(_: SubscriptOutOfRangeError) -> Self {
        Self::SubscriptOutOfRange
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_code() {
        let errors = [
            RuntimeError::ReturnWithoutGoSub,
            RuntimeError::IllegalFunctionCall,
            RuntimeError::Overflow,
            RuntimeError::SubscriptOutOfRange,
            RuntimeError::DivisionByZero,
            RuntimeError::TypeMismatch,
            RuntimeError::ResumeWithoutError,
            RuntimeError::BadFileNameOrNumber,
            RuntimeError::FileNotFound,
            RuntimeError::FileAlreadyOpen,
            RuntimeError::InputPastEndOfFile,
            // the following are not qbasic codes
            RuntimeError::Other("whatever".to_owned()),
            RuntimeError::ForLoopZeroStep,
        ];
        let codes = [3, 5, 6, 9, 11, 13, 20, 52, 53, 55, 62, 257, 258];

        assert_eq!(errors.len(), codes.len());
        for i in 0..errors.len() {
            let error = &errors[i];
            let code = codes[i];
            assert_eq!(error.get_code(), code);
        }
    }
}
