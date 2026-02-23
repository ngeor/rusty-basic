use rusty_parser::{BareName, Name};

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    /// The resolved qualified name of a function.
    Function(Name),

    /// The resolved name of a sub.
    Sub(BareName),
}
