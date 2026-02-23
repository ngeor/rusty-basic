use rusty_parser::{BareName, Name};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    Global,
    Sub,
    Function,
}

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ScopeName {
    /// The global scope.
    Global,

    /// The resolved qualified name of a function.
    Function(Name),

    /// The resolved name of a sub.
    Sub(BareName),
}

impl From<&ScopeName> for ScopeKind {
    fn from(scope_name: &ScopeName) -> Self {
        match scope_name {
            ScopeName::Global => Self::Global,
            ScopeName::Function(_) => Self::Function,
            ScopeName::Sub(_) => Self::Sub,
        }
    }
}
