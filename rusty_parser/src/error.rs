use rusty_common::Positioned;

/// Represents parser errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
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

    #[deprecated]
    Incomplete,

    Expected(String),

    Failure,

    LoopWithoutDo,
}

impl ParseError {
    pub fn syntax_error(msg: &str) -> Self {
        ParseError::SyntaxError(msg.to_string())
    }

    pub fn expected(msg: &str) -> Self {
        ParseError::Expected(msg.to_owned())
    }
}

pub trait ParserErrorTrait {
    fn is_incomplete(&self) -> bool;

    fn no_incomplete(self) -> Self;
}

impl ParserErrorTrait for ParseError {
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

pub type ParseErrorPos = Positioned<ParseError>;

impl From<std::io::Error> for ParseError {
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

impl From<std::num::ParseFloatError> for ParseError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::ParseNumError(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseNumError(e.to_string())
    }
}
