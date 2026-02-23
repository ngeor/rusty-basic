use std::collections::HashMap;

use rusty_common::Positioned;
use rusty_parser::{BareName, TypeQualifier};

use crate::core::ResolvedParamTypes;

/// The signature of a FUNCTION or SUB.
/// Consists of the resolved parameter types and, in case of a FUNCTION, the return type.
#[derive(PartialEq)]
pub enum Signature {
    /// The signature of a FUNCTION consists of the resolved parameter types
    /// and the return type.
    Function(ResolvedParamTypes, TypeQualifier),

    /// The signature of a SUB consists of the resolved parameter types.
    Sub(ResolvedParamTypes),
}

impl Signature {
    pub fn qualifier(&self) -> Option<TypeQualifier> {
        match self {
            Self::Function(_, q) => Some(*q),
            Self::Sub(_) => None,
        }
    }

    pub fn param_types(&self) -> &ResolvedParamTypes {
        match self {
            Self::Function(param_types, _) => param_types,
            Self::Sub(param_types) => param_types,
        }
    }
}

// Equality between a Signature and a TypeQualifier.
// Equality is done against the return type of the Signature,
// which always fails for a SUB.

impl PartialEq<TypeQualifier> for Signature {
    fn eq(&self, other: &TypeQualifier) -> bool {
        if let Self::Function(_, q) = self
            && q == other
        {
            true
        } else {
            false
        }
    }
}

/// A map of (bare) subprogram names to their respective signatures.
pub type SignatureMap = HashMap<BareName, Positioned<Signature>>;
