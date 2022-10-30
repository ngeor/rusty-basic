use crate::converter::convert;
use crate::post_linter::post_linter;
use crate::pre_linter::pre_lint_program;
use crate::HasUserDefinedTypes;
use rusty_common::QErrorNode;
use rusty_parser::ProgramNode;

pub fn lint(program: ProgramNode) -> Result<(ProgramNode, impl HasUserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = pre_lint_program(&program)?;
    // convert to fully typed
    let (pre_linter_result, program) = convert(program, pre_linter_result)?;
    // lint and reduce
    post_linter(program, &pre_linter_result).map(|program_node| (program_node, pre_linter_result))
}
