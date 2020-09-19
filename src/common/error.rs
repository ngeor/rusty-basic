use super::ErrorEnvelope;

#[derive(Clone, Debug, PartialEq)]
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

    // Lexer errors
    UnsupportedCharacter(char),

    // General fallback
    Other(String),
}

impl QError {
    pub fn syntax_error<S: AsRef<str>>(msg: S) -> Self {
        QError::SyntaxError(format!("{}", msg.as_ref()))
    }

    pub fn syntax_error_fn<S: AsRef<str>>(msg: S) -> impl Fn() -> QError {
        // repeating of format due to "cannot move out of closure"
        move || QError::syntax_error(format!("{}", msg.as_ref()))
    }

    pub fn syntax_error_fn_fn<S: AsRef<str> + 'static>(
        msg: S,
    ) -> impl Fn() -> Box<dyn Fn() -> QError> {
        // repeating of format due to "cannot move out of closure"
        move || Box::new(QError::syntax_error_fn(format!("{}", msg.as_ref())))
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
