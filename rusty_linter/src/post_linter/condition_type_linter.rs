use crate::core::{LintError, LintErrorPos};
use crate::post_linter::post_conversion_linter::PostConversionLinter;
use rusty_common::AtPos;
use rusty_parser::specific::{
    ConditionalBlock, DoLoop, ExpressionPos, ExpressionType, HasExpressionType, TypeQualifier,
};

/// Ensures that expressions appearing in logical conditions are numeric.
pub struct ConditionTypeLinter {}

impl ConditionTypeLinter {
    fn ensure_expression_is_condition(expr_pos: &ExpressionPos) -> Result<(), LintErrorPos> {
        match expr_pos.expression_type() {
            ExpressionType::BuiltIn(q) => {
                if q == TypeQualifier::DollarString {
                    Err(LintError::TypeMismatch.at(expr_pos))
                } else {
                    Ok(())
                }
            }
            _ => Err(LintError::TypeMismatch.at(expr_pos)),
        }
    }
}

impl PostConversionLinter for ConditionTypeLinter {
    fn visit_conditional_block(&mut self, c: &ConditionalBlock) -> Result<(), LintErrorPos> {
        self.visit_statements(&c.statements)?;
        Self::ensure_expression_is_condition(&c.condition)
    }

    fn visit_do_loop(&mut self, do_loop: &DoLoop) -> Result<(), LintErrorPos> {
        self.visit_statements(&do_loop.statements)?;
        Self::ensure_expression_is_condition(&do_loop.condition)
    }
}
