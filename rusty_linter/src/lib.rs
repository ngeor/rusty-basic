mod built_ins;
mod converter;
mod core;
mod names;
mod post_linter;
mod pre_linter;
#[cfg(test)]
mod tests;

pub use self::core::{
    qualifier_of_variant, CastVariant, HasUserDefinedTypes, LintError, QBNumberCast, SubprogramName,
};

use crate::converter::convert;
use crate::core::LintErrorPos;
use crate::post_linter::post_linter;
use crate::pre_linter::pre_lint_program;
use rusty_parser::specific::Program;

pub fn lint(program: Program) -> Result<(Program, impl HasUserDefinedTypes), LintErrorPos> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = pre_lint_program(&program)?;
    // convert to fully typed
    let (context, program) = convert(program, pre_linter_result)?;
    // lint and reduce
    post_linter(program, &context).map(|program| (program, context))
}
