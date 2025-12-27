use std::collections::HashMap;

use rusty_common::Positioned;
use rusty_parser::{BareName, BuiltInStyle, Name, TypeQualifier};

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    /// The resolved qualified name of a function.
    Function(Name),

    /// The resolved name of a sub.
    Sub(BareName),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameContext {
    Global,
    Sub,
    Function,
}

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

/// The signature of a FUNCTION or SUB.
/// Consists of the resolved parameter types and, in case of a FUNCTION, the return type.
#[derive(PartialEq)]
pub struct Signature {
    /// The return type of a FUNCTION, or None if this is the signature of a SUB.
    q: Option<TypeQualifier>,

    /// The resolved parameter types.
    param_types: ResolvedParamTypes,
}

impl Signature {
    pub fn new_sub(param_types: ResolvedParamTypes) -> Self {
        Self {
            q: None,
            param_types,
        }
    }

    pub fn new_function(q: TypeQualifier, param_types: ResolvedParamTypes) -> Self {
        Self {
            q: Some(q),
            param_types,
        }
    }

    pub fn qualifier(&self) -> Option<TypeQualifier> {
        self.q.as_ref().copied()
    }

    pub fn param_types(&self) -> &ResolvedParamTypes {
        &self.param_types
    }
}

// Equality between a Signature and a TypeQualifier.
// Equality is done against the return type of the Signature,
// which always fails for a SUB.

impl PartialEq<TypeQualifier> for Signature {
    fn eq(&self, other: &TypeQualifier) -> bool {
        match &self.q {
            Some(q) => q == other,
            _ => false,
        }
    }
}

/// A map of (bare) subprogram names to their respective signatures.
pub type SignatureMap = HashMap<BareName, Positioned<Signature>>;
