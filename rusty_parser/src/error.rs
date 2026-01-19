use rusty_common::Positioned;
use rusty_pc::ParserErrorTrait;

/// Represents parser errors.
/// All errors except `Miss` and `Expected` are fatal.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ParserError {
    /// Indicates a generic miss in parsing (soft error).
    #[default]
    Miss,

    /// A soft error with a description of what was expected.
    Expected(String),

    // 1
    NextWithoutFor,

    // 2
    SyntaxError(String),

    // 6
    Overflow,

    // 26
    ForWithoutNext,

    // 29
    WhileWithoutWend,

    // 30
    WendWithoutWhile,

    ParseNumError(String),

    // 52
    BadFileNameOrNumber,

    // 53
    FileNotFound,

    // 57
    DeviceIOError(String),

    // 62
    InputPastEndOfFile,

    ElseWithoutIf,

    IdentifierCannotIncludePeriod,

    IdentifierTooLong,

    ElementNotDefined,

    LoopWithoutDo,
}

impl ParserError {
    pub fn syntax_error(msg: &str) -> Self {
        Self::SyntaxError(msg.to_string())
    }

    /// Creates a syntax error that starts with "Expected: "
    /// followed by the given string.
    pub fn expected(expectation: &str) -> Self {
        Self::Expected(format!("Expected: {}", expectation))
    }
}

impl ParserErrorTrait for ParserError {
    fn is_soft(&self) -> bool {
        matches!(self, Self::Miss | Self::Expected(_))
    }

    fn is_fatal(&self) -> bool {
        !self.is_soft()
    }

    fn to_fatal(self) -> Self {
        match self {
            Self::Expected(msg) => Self::SyntaxError(msg),
            Self::Miss => Self::SyntaxError("Unknown error".to_string()),
            _ => self,
        }
    }
}

pub type ParseErrorPos = Positioned<ParserError>;

impl From<std::num::ParseFloatError> for ParserError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::ParseNumError(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ParserError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseNumError(e.to_string())
    }
}
