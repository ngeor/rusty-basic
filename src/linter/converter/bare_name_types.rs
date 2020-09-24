use crate::parser::{BareName, TypeQualifier};
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum BareNameTypes {
    /// DIM X, DIM X$, X = 42, etc
    Compact(HashSet<TypeQualifier>),
    /// CONST X = 42
    Constant(TypeQualifier),
    /// DIM X AS STRING * 5
    FixedLengthString(u16),
    /// DIM X AS INTEGER
    Extended(TypeQualifier),
    /// DIM X AS Card
    UserDefined(BareName),
}

impl BareNameTypes {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            _ => false,
        }
    }

    pub fn is_extended(&self) -> bool {
        match self {
            Self::FixedLengthString(_) | Self::Extended(_) | Self::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn has_compact(&self, q: TypeQualifier) -> bool {
        match self {
            Self::Compact(qualifiers) => qualifiers.contains(&q),
            _ => false,
        }
    }

    pub fn add_compact(&mut self, q: TypeQualifier) {
        match self {
            Self::Compact(qualifiers) => {
                if !qualifiers.insert(q) {
                    panic!("Duplicate compact qualifier");
                }
            }
            _ => panic!("Cannot add compact to this set"),
        }
    }
}
