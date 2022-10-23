use super::ErrorEnvelope;

/// Represents QBasic errors.
/// Note that for convenience this enum also holds a few custom errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QError {
    // 1
    NextWithoutFor,

    // 2
    SyntaxError(String),

    // 3
    ReturnWithoutGoSub,

    // 4
    OutOfData,

    // 5
    IllegalFunctionCall,

    // 6
    Overflow,

    // 7
    OutOfMemory,

    // 8
    LabelNotDefined,

    // 9
    SubscriptOutOfRange,

    // 10
    DuplicateDefinition,

    // 11
    DivisionByZero,

    // 12
    IllegalInDirectMode,

    // 13
    TypeMismatch,

    // 14
    OutOfStringSpace,

    // 16
    StringFormulaTooComplex,

    // 17
    CannotContinue,

    // 18
    FunctionNotDefined,

    // 19
    NoResume,

    // 20
    ResumeWithoutError,

    // 24
    DeviceTimeout,

    // 25
    DeviceFault,

    // 26
    ForWithoutNext,

    // 27
    OutOfPaper,

    // 29
    WhileWithoutWend,

    // 30
    WendWithoutWhile,

    // 33
    DuplicateLabel,

    // 35
    SubprogramNotDefined,

    // 37
    ArgumentCountMismatch,

    // 38
    ArrayNotDefined,

    // 40
    VariableRequired,

    // 50
    FieldOverflow,

    // 51
    InternalError(String),

    // 52
    BadFileNameOrNumber,

    // 53
    FileNotFound,

    // 54
    BadFileMode,

    // 55
    FileAlreadyOpen,

    // 56
    FieldStatementActive,

    // 57
    DeviceIOError(String),

    // 58
    FileAlreadyExists,

    // 59
    BadRecordLength,

    // 61
    DiskFull,

    // 62
    InputPastEndOfFile,

    // 63
    BadRecordNumber,

    // 64
    BadFileName,

    // 67
    TooManyFiles,

    // 68
    DeviceUnavailable,

    // 69
    CommunicationBufferOverflow,

    // 70
    PermissionDenied,

    // 71
    DiskNotReady,

    // 72
    DiskMediaError,

    // 73
    FeatureUnavailable,

    // 74
    RenameAcrossDisks,

    // 75
    PathFileAccessError,

    // 76
    PathNotFound,

    //
    // The following are not standard errors
    //
    ArgumentTypeMismatch,
    InvalidConstant,

    ForLoopZeroStep,

    UnterminatedIf,
    UnterminatedElse,
    ElseWithoutIf,

    /// Not a standard error, thrown by the linter to indicate that an identifier
    /// that contains a period is clashing with a variable of a user defined type.
    ///
    /// e.g. DIM A.B AS STRING would clash with A if A were a variable of user defined type.
    ///
    /// QBasic throws various syntax errors on this edge case, depending on the identifier type,
    /// the order of declaration, etc.
    DotClash,

    IdentifierCannotIncludePeriod,

    IdentifierTooLong,

    TypeNotDefined,

    ElementNotDefined,

    IllegalInSubFunction,

    FunctionNeedsArguments,

    /// Indicates that a REDIM statement is trying to change a variable
    /// previously defined with DIM.
    ArrayAlreadyDimensioned,

    /// Indicates that a REDIM statement is trying to change the number of
    /// dimensions previously defined in an array.
    WrongNumberOfDimensions,

    // Lexer errors
    UnsupportedCharacter(char),

    // General fallback
    Other(String),

    // Parser errors
    Incomplete,
    Expected(String),
    Failure,
}

impl QError {
    pub fn syntax_error<S: AsRef<str>>(msg: S) -> Self {
        QError::SyntaxError(msg.as_ref().to_string())
    }

    pub fn expected(msg: &str) -> Self {
        QError::Expected(msg.to_owned())
    }

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
            Self::InternalError(_) => 256,
            Self::Other(_) => 257,
            Self::ForLoopZeroStep => 258,
            _ => todo!(),
        }
    }
}

pub trait ParserErrorTrait {
    fn is_incomplete(&self) -> bool;

    fn no_incomplete(self) -> Self;
}

impl ParserErrorTrait for QError {
    fn is_incomplete(&self) -> bool {
        matches!(self, Self::Incomplete | Self::Expected(_))
    }

    fn no_incomplete(self) -> Self {
        match self {
            Self::Incomplete => Self::Failure,
            Self::Expected(s) => Self::SyntaxError(s),
            _ => self,
        }
    }
}

impl<T, E: ParserErrorTrait> ParserErrorTrait for Result<T, E> {
    fn is_incomplete(&self) -> bool {
        matches!(self, Err(err) if err.is_incomplete())
    }

    fn no_incomplete(self) -> Self {
        self.map_err(|err| err.no_incomplete())
    }
}

pub type QErrorNode = ErrorEnvelope<QError>;

impl From<&std::io::Error> for QError {
    fn from(e: &std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::FileNotFound
        } else if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::InputPastEndOfFile
        } else {
            Self::DeviceIOError(e.to_string())
        }
    }
}

impl From<std::io::Error> for QError {
    fn from(e: std::io::Error) -> Self {
        let x: &std::io::Error = &e;
        x.into()
    }
}

impl From<std::num::ParseFloatError> for QError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::InternalError(e.to_string())
    }
}

impl From<std::num::ParseIntError> for QError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::InternalError(e.to_string())
    }
}

impl From<&str> for QError {
    fn from(x: &str) -> Self {
        x.to_string().into()
    }
}

impl From<String> for QError {
    fn from(x: String) -> Self {
        Self::Other(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_code() {
        let errors = [
            QError::ReturnWithoutGoSub,
            QError::IllegalFunctionCall,
            QError::Overflow,
            QError::SubscriptOutOfRange,
            QError::DivisionByZero,
            QError::TypeMismatch,
            QError::ResumeWithoutError,
            QError::BadFileNameOrNumber,
            QError::FileNotFound,
            QError::FileAlreadyOpen,
            QError::InputPastEndOfFile,
            // the following are not qbasic codes
            QError::InternalError("whatever".to_owned()),
            QError::Other("whatever".to_owned()),
            QError::ForLoopZeroStep,
        ];
        let codes = [3, 5, 6, 9, 11, 13, 20, 52, 53, 55, 62, 256, 257, 258];

        assert_eq!(errors.len(), codes.len());
        for i in 0..errors.len() {
            let error = &errors[i];
            let code = codes[i];
            assert_eq!(error.get_code(), code);
        }
    }
}
