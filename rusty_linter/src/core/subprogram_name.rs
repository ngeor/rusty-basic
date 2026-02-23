use rusty_parser::{BareName, Name};

use crate::core::NameScope;

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    /// The global scope.
    Global,

    /// The resolved qualified name of a function.
    Function(Name),

    /// The resolved name of a sub.
    Sub(BareName),
}

impl From<&SubprogramName> for NameScope {
    fn from(subprogram_name: &SubprogramName) -> Self {
        match subprogram_name {
            SubprogramName::Global => Self::Global,
            SubprogramName::Function(_) => Self::Function,
            SubprogramName::Sub(_) => Self::Sub,
        }
    }
}
