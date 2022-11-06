//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod context;
mod dim_rules;
mod expr_rules;
mod names;
mod pos_context;
mod program_rules;
mod statement;
mod traits;
mod types;

use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::pre_linter::PreLinterResult;
use crate::LintErrorPos;
use rusty_parser::Program;

pub fn convert(
    program: Program,
    pre_linter_result: PreLinterResult,
) -> Result<(PreLinterResult, Program), LintErrorPos> {
    let mut context = Context::new(pre_linter_result);
    program
        .convert(&mut context)
        .map(|p| (context.pre_linter_result(), p))
}
