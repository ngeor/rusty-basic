use super::ErrorEnvelope;

// TODO clarify when we use Unexpected | Unterminated or SyntaxError
// TODO go over all sub-parsers and make sure they honor those semantics and don't backtrack if they aren't supposed do
// TODO add tests for more user friendly errors e.g. "ELSE without IF"

#[derive(Clone, Debug, PartialEq)]
pub enum QError {
    // 37
    ArgumentCountMismatch,

    ArgumentTypeMismatch,

    // 13
    TypeMismatch,

    // 1
    NextWithoutFor,

    // 10
    DuplicateDefinition,

    InvalidConstant,

    // 35
    SubprogramNotDefined,

    // 8
    LabelNotDefined,

    // 33
    DuplicateLabel,

    // 40
    VariableRequired,

    // 2
    SyntaxError(String),

    ForLoopZeroStep,
    Overflow,
    FileNotFound,
    IllegalFunctionCall,
    DivisionByZero,
    IO(String),
    Other(String),
}

pub type QErrorNode = ErrorEnvelope<QError>;

impl From<std::io::Error> for QError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::FileNotFound
        } else {
            Self::IO(e.to_string())
        }
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
