use rusty_common::Positioned;
use rusty_pc::ParserErrorTrait;

/// Represents parser errors.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ParserError {
    fatal: bool,
    kind: ParserErrorKind,
}

impl ParserError {
    pub fn soft(kind: ParserErrorKind) -> Self {
        Self { fatal: false, kind }
    }

    pub fn fatal(kind: ParserErrorKind) -> Self {
        Self { fatal: true, kind }
    }

    pub fn kind(&self) -> &ParserErrorKind {
        &self.kind
    }

    pub fn to_kind(self) -> ParserErrorKind {
        self.kind
    }

    /// Creates a soft syntax error that starts with "Expected: "
    /// followed by the given string.
    pub fn expected(expectation: &str) -> Self {
        Self::soft(ParserErrorKind::expected(expectation))
    }

    /// Creates a fatal syntax error.
    pub fn syntax_error(msg: &str) -> Self {
        Self::fatal(ParserErrorKind::syntax_error(msg))
    }
}

impl ParserErrorTrait for ParserError {
    fn is_fatal(&self) -> bool {
        self.fatal
    }

    fn to_fatal(self) -> Self {
        Self {
            fatal: true,
            ..self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ParserErrorKind {
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
    #[default]
    InputPastEndOfFile,

    ElseWithoutIf,

    IdentifierCannotIncludePeriod,

    IdentifierTooLong,

    ElementNotDefined,

    LoopWithoutDo,
}

impl ParserErrorKind {
    pub fn syntax_error(msg: &str) -> Self {
        Self::SyntaxError(msg.to_string())
    }

    /// Creates a syntax error that starts with "Expected: "
    /// followed by the given string.
    pub fn expected(expectation: &str) -> Self {
        Self::SyntaxError(format!("Expected: {}", expectation))
    }
}

pub type ParseErrorPos = Positioned<ParserError>;

impl From<&str> for ParserErrorKind {
    fn from(s: &str) -> Self {
        Self::SyntaxError(format!("Expected: {}", s))
    }
}

impl From<String> for ParserErrorKind {
    fn from(s: String) -> Self {
        Self::SyntaxError(format!("Expected: {}", s))
    }
}

impl From<std::num::ParseFloatError> for ParserError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::fatal(ParserErrorKind::ParseNumError(e.to_string()))
    }
}

impl From<std::num::ParseIntError> for ParserError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::fatal(ParserErrorKind::ParseNumError(e.to_string()))
    }
}
