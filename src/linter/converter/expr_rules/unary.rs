use crate::linter::converter::expr_rules::*;

pub fn convert(
    ctx: &mut PosExprState,
    unary_operator: UnaryOperator,
    child: ExpressionNode,
) -> Result<Expression, QErrorNode> {
    // convert child (recursion)
    let converted_child = child.convert(ctx)?;
    // ensure operator applies to converted expr
    let converted_expr_type = converted_child.as_ref().expression_type();
    if is_applicable_to_expr_type(&converted_expr_type) {
        let unary_expr = Expression::UnaryExpression(unary_operator, Box::new(converted_child));
        Ok(unary_expr)
    } else {
        Err(QError::TypeMismatch).with_err_at(&converted_child)
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
