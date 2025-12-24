//! Contains simple types and type aliases.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DimContext {
    /// Normal DIM statement
    #[default]
    Default,

    /// REDIM statement
    Redim,
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
