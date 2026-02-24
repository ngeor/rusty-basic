use rusty_common::*;
use rusty_parser::SubCall;

use super::post_conversion_linter::PostConversionLinter;
use super::user_defined_function_linter::lint_call_args;
use crate::Context;
use crate::core::{LintError, LintErrorPos};

pub struct UserDefinedSubLinter<'a> {
    pub linter_context: &'a Context,
}

impl<'a> PostConversionLinter for UserDefinedSubLinter<'a> {
    fn visit_sub_call(&mut self, sub_call: &SubCall, pos: Position) -> Result<(), LintErrorPos> {
        let (name, args) = sub_call.into();
        match self.linter_context.subs.get(name) {
            Some(sub_signature_pos) => {
                lint_call_args(args, sub_signature_pos.element.param_types(), pos)
            }
            None => Err(LintError::SubprogramNotDefined.at_pos(pos)),
        }
    }
}
