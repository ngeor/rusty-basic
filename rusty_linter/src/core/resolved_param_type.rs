use rusty_parser::{BareName, BuiltInStyle, TypeQualifier};

/// A resolved parameter type.
#[derive(PartialEq)]
pub enum ResolvedParamType {
    /// A built-in type.
    /// The type qualifier indicates the type.
    /// The style indicates how the parameter was declared:
    /// Compact: e.g. `A$` or Extended e.g. `A AS STRING`
    BuiltIn(TypeQualifier, BuiltInStyle),

    /// A user defined type.
    UserDefined(BareName),

    /// An array type.
    /// Dimensions are not allowed for parameter types.
    Array(Box<Self>),
}

/// A collection of resolved parameter types.
pub type ResolvedParamTypes = Vec<ResolvedParamType>;
