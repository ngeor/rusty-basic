//! Contains simple types and type aliases.

use crate::parser::QualifiedNameNode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DimContext {
    /// Normal DIM statement
    Default,

    /// REDIM statement
    Redim,
}

impl Default for DimContext {
    fn default() -> Self {
        Self::Default
    }
}

/// Indicates the context in which an expression is being resolved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    /// Default context (typically r-side expression)
    Default,

    /// Assignment (typically l-side expression)
    Assignment,

    /// Function or sub argument
    Argument,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

impl Default for ExprContext {
    fn default() -> Self {
        Self::Default
    }
}

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
pub type Implicits = Vec<QualifiedNameNode>;
