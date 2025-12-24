use crate::pre_linter::{FunctionSignature, SubSignature};
use rusty_common::Positioned;
use rusty_parser::{BareName, BuiltInStyle, QualifiedName, TypeQualifier};
use std::collections::HashMap;

pub type SubMap = HashMap<BareName, Positioned<SubSignature>>;
pub type FunctionMap = HashMap<BareName, Positioned<FunctionSignature>>;

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

#[derive(Eq, PartialEq)]
pub enum ResolvedParamType {
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareName),
    Array(Box<Self>),
}
