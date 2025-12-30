mod built_ins;
mod converter;
mod core;
mod names;
mod post_linter;
mod pre_linter;
#[cfg(test)]
mod tests;

use rusty_parser::Program;

pub use self::converter::Context;
pub use self::core::{
    qualifier_of_variant, CastVariant, HasUserDefinedTypes, LintError, QBNumberCast, SubprogramName
};
pub use self::names::Names;
use crate::converter::convert;
use crate::core::LintErrorPos;
use crate::post_linter::post_linter;
use crate::pre_linter::pre_lint_program;

pub fn lint(program: Program) -> Result<(Program, Context), LintErrorPos> {
    // first pass, get user defined types and functions/subs
    let pre_linter_result = pre_lint_program(&program)?;
    // convert to fully typed
    let (context, program) = convert(program, pre_linter_result)?;
    // lint and reduce
    post_linter(program, &context).map(|program| (program, context))
}
