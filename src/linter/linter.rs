use super::types::*;
use crate::common::*;
use crate::linter::converter::convert;
use crate::linter::post_linter::post_linter;
use crate::linter::pre_linter::subprogram_context::parse_subprograms_and_types;
use crate::parser;

pub fn lint(program: parser::ProgramNode) -> Result<(ProgramNode, UserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let (functions, subs, user_defined_types) = parse_subprograms_and_types(&program)?;
    // convert to fully typed
    let (result, names_without_dot) = convert(program, &functions, &subs, &user_defined_types)?;
    // lint and reduce
    post_linter(result, &functions, &subs, &names_without_dot).map(|p| (p, user_defined_types))
}
