use crate::core::{ConstLookup, LintResult};
use crate::core::{LintError, LintErrorPos};
use rusty_common::Positioned;
use rusty_parser::{Expression, ExpressionPos, TypeQualifier};
use rusty_variant::{Variant, MAX_INTEGER};

pub fn validate_string_length(
    expr_pos: &ExpressionPos,
    lookup: &impl ConstLookup,
) -> Result<u16, LintErrorPos> {
    let Positioned { element: expr, pos } = expr_pos;
    do_validate_string_length(expr, lookup).with_err_at(pos)
}

fn do_validate_string_length(
    expr: &Expression,
    lookup: &impl ConstLookup,
) -> Result<u16, LintError> {
    match expr {
        Expression::IntegerLiteral(i) => {
            if (1..=MAX_INTEGER).contains(i) {
                Ok(*i as u16)
            } else {
                Err(LintError::InvalidConstant)
            }
        }
        Expression::Variable(name, _) => {
            if let Some(qualifier) = name.qualifier() {
                if qualifier != TypeQualifier::PercentInteger {
                    return Err(LintError::InvalidConstant);
                }
            }

            if let Some(Variant::VInteger(i)) = lookup.get_resolved_constant(name.bare_name()) {
                if (1..=MAX_INTEGER).contains(i) {
                    return Ok(*i as u16);
                }
            }

            Err(LintError::InvalidConstant)
        }
        _ => Err(LintError::InvalidConstant),
    }
}
