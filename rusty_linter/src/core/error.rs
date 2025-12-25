use rusty_common::{AtPos, HasPos, Positioned};
use rusty_parser::ParseError;
use rusty_variant::VariantError;

#[derive(Clone, Debug, PartialEq)]
pub enum LintError {
    ArgumentCountMismatch,
    ArgumentTypeMismatch,
    ArrayAlreadyDimensioned,
    ArrayNotDefined,
    DivisionByZero,
    DotClash,
    DuplicateDefinition,
    DuplicateLabel,
    ElementNotDefined,
    FunctionNeedsArguments,
    IllegalInSubFunction,
    IllegalOutsideSubFunction,
    InvalidConstant,
    LabelNotDefined,
    NextWithoutFor,
    OutOfStringSpace,
    Overflow,
    SubprogramNotDefined,
    TypeMismatch,
    TypeNotDefined,
    VariableRequired,
    WrongNumberOfDimensions,
    ParserError(ParseError),

    // custom
    NotFiniteNumber,
}

pub type LintErrorPos = Positioned<LintError>;

impl From<VariantError> for LintError {
    fn from(e: VariantError) -> Self {
        match e {
            VariantError::DivisionByZero => Self::DivisionByZero,
            VariantError::Overflow => Self::Overflow,
            VariantError::TypeMismatch => Self::TypeMismatch,
        }
    }
}

impl From<ParseError> for LintError {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::NextWithoutFor => Self::NextWithoutFor,
            ParseError::Overflow => Self::Overflow,
            ParseError::ElementNotDefined => Self::ElementNotDefined,
            _ => Self::ParserError(e),
        }
    }
}

pub trait LintResult<T> {
    fn with_err_at(self, pos: &impl HasPos) -> Result<T, LintErrorPos>;
}

impl<T> LintResult<T> for Result<T, LintError> {
    fn with_err_at(self, pos: &impl HasPos) -> Result<T, LintErrorPos> {
        self.map_err(|e| e.at(pos))
    }
}
