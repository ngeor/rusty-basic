//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod common;
mod dim_rules;
mod expr_rules;
mod statement;

use rusty_parser::Program;

use crate::Context;
use crate::converter::common::Convertible;
use crate::core::LintErrorPos;
use crate::pre_linter::PreLinterResult;

pub fn convert(
    program: Program,
    pre_linter_result: PreLinterResult,
) -> Result<(Context, Program), LintErrorPos> {
    let mut context = Context::new(pre_linter_result);
    program.convert(&mut context).map(|p| (context, p))
}
