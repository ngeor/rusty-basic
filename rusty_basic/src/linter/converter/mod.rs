//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod context;
mod dim_rules;
mod expr_rules;
mod names;
mod pos_context;
mod statement;
mod top_level_rules;
mod traits;
mod types;

use crate::linter::converter::context::Context;
use crate::linter::converter::traits::Convertible;
use crate::linter::pre_linter::PreLinterResult;
use rusty_common::QErrorNode;
use rusty_parser::ProgramNode;

pub fn convert(
    program: ProgramNode,
    pre_linter_result: PreLinterResult,
) -> Result<(PreLinterResult, ProgramNode), QErrorNode> {
    let mut context = Context::new(pre_linter_result);
    program
        .convert(&mut context)
        .map(|p| (context.pre_linter_result(), p))
}
