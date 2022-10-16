use crate::common::{Locatable, QErrorNode};
use crate::linter::converter::convert;
use crate::linter::post_linter::post_linter;
use crate::linter::pre_linter::pre_lint_program;
use crate::linter::HasUserDefinedTypes;
use crate::parser::{BareName, ParamType, ProgramNode, QualifiedName, TypeQualifier};
use std::collections::HashMap;
use std::rc::Rc;

pub fn lint(program: ProgramNode) -> Result<(ProgramNode, impl HasUserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = Rc::new(pre_lint_program(&program)?);
    // convert to fully typed
    let (result, names_without_dot) = convert(program, Rc::clone(&pre_linter_result))?;
    // lint and reduce
    post_linter(result, &pre_linter_result, &names_without_dot)
        .map(|program_node| (program_node, pre_linter_result))
}

pub type ParamTypes = Vec<ParamType>;

pub type SubSignature = ParamTypes;
pub type SubSignatureNode = Locatable<SubSignature>;
pub type SubMap = HashMap<BareName, SubSignatureNode>;

pub type FunctionSignature = (TypeQualifier, ParamTypes);
pub type FunctionSignatureNode = Locatable<FunctionSignature>;
pub type FunctionMap = HashMap<BareName, FunctionSignatureNode>;

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
