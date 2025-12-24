//! Contains simple types and type aliases.
use rusty_common::{Position, Positioned};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DimContext {
    /// Normal DIM statement
    #[default]
    Default,

    /// REDIM statement
    Redim,
}

pub struct DimNameState {
    pub dim_context: DimContext,
    pub shared: bool,
    pub pos: Position,
}

/// Indicates the context in which an expression is being resolved.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ExprContext {
    /// Default context (typically r-side expression)
    #[default]
    Default,

    /// Assignment (typically l-side expression)
    Assignment,

    /// Function or sub argument
    Argument,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

pub type ExprContextPos = Positioned<ExprContext>;
