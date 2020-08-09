use crate::common::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinterError {
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
    SyntaxError,
}

pub type Error = ErrorEnvelope<LinterError>;
