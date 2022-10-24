use crate::common::Locatable;
use crate::linter::pre_linter::{FunctionSignature, SubSignature};
use crate::parser::{BareName, QualifiedName};
use std::collections::HashMap;

pub type SubMap = HashMap<BareName, Locatable<SubSignature>>;
pub type FunctionMap = HashMap<BareName, Locatable<FunctionSignature>>;

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    /// The resolved name of a function.
    Function(QualifiedName),

    /// The resolved name of a sub.
    Sub(BareName),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameContext {
    Global,
    Sub,
    Function,
}
