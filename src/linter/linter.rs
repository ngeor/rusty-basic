use crate::common::QErrorNode;
use crate::linter::converter::convert;
use crate::linter::post_linter::post_linter;
use crate::linter::pre_linter::pre_lint_program;
use crate::linter::HasUserDefinedTypes;
use crate::parser::ProgramNode;

pub fn lint(program: ProgramNode) -> Result<(ProgramNode, impl HasUserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = pre_lint_program(&program)?;
    // convert to fully typed
    let (pre_linter_result, program) = convert(program, pre_linter_result)?;
    // lint and reduce
    post_linter(program, &pre_linter_result).map(|program_node| (program_node, pre_linter_result))
}
