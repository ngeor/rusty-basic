use rusty_parser::Program;

use crate::converter::common::Convertible;
use crate::core::{LintErrorPos, LinterContext};
use crate::post_linter::post_linter;
use crate::pre_linter::pre_lint_program;

pub fn lint(program: Program) -> Result<(Program, LinterContext), LintErrorPos> {
    // first pass, get user defined types and functions/subs
    let mut context = pre_lint_program(&program)?;
    // convert to fully typed
    let program = program.convert(&mut context)?;
    // lint and reduce
    post_linter(program, &context).map(|program| (program, context))
}
