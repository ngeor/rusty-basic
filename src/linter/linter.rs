use crate::common::QErrorNode;
use crate::linter::converter::convert;
use crate::linter::post_linter::post_linter;
use crate::linter::pre_linter::pre_lint_program;
use crate::linter::HasUserDefinedTypes;
use crate::parser::ProgramNode;
use std::rc::Rc;

// TODO remove all uses of Rc and RefCell in the codebase

pub fn lint(program: ProgramNode) -> Result<(ProgramNode, impl HasUserDefinedTypes), QErrorNode> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = Rc::new(pre_lint_program(&program)?);
    // convert to fully typed
    let (result, names_without_dot) = convert(program, Rc::clone(&pre_linter_result))?;
    // lint and reduce
    post_linter(result, &pre_linter_result, &names_without_dot)
        .map(|program_node| (program_node, pre_linter_result))
}
