use crate::converter::expr_rules::*;
use crate::error::{LintError, LintErrorPos};

pub fn convert(
    ctx: &mut PosExprState,
    unary_operator: UnaryOperator,
    child: ExpressionPos,
) -> Result<Expression, LintErrorPos> {
    // convert child (recursion)
    let converted_child = child.convert(ctx)?;
    // ensure operator applies to converted expr
    let converted_expr_type = converted_child.expression_type();
    if is_applicable_to_expr_type(&converted_expr_type) {
        let unary_expr = Expression::UnaryExpression(unary_operator, Box::new(converted_child));
        Ok(unary_expr)
    } else {
        Err(LintError::TypeMismatch).with_err_at(&converted_child)
    }
}

fn is_applicable_to_expr_type(expr_type: &ExpressionType) -> bool {
    matches!(
        expr_type,
        ExpressionType::BuiltIn(TypeQualifier::BangSingle)
            | ExpressionType::BuiltIn(TypeQualifier::HashDouble)
            | ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            | ExpressionType::BuiltIn(TypeQualifier::AmpersandLong)
    )
}
