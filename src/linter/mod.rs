pub mod arg_validation;
mod const_value_resolver;
mod converter;
mod post_linter;
mod pre_linter;
mod type_resolver;
mod type_resolver_impl;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;

use crate::common::QErrorNode;
use crate::linter::converter::convert;
use crate::linter::pre_linter::subprogram_context::parse_subprograms_and_types;
use crate::parser::{BareName, ProgramNode, QualifiedName, UserDefinedTypes};

pub fn lint(program: ProgramNode) -> Result<(ProgramNode, UserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let (functions, subs, user_defined_types) = parse_subprograms_and_types(&program)?;
    // convert to fully typed
    let (result, names_without_dot) = convert(program, &functions, &subs, &user_defined_types)?;
    // lint and reduce
    post_linter::post_linter(result, &functions, &subs, &names_without_dot)
        .map(|p| (p, user_defined_types))
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DimContext {
    /// Normal DIM statement
    Default,

    /// REDIM statement
    Redim,

    /// A function/sub parameter
    Param,
}
