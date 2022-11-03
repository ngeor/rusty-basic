use rusty_common::*;
use rusty_parser::{Expression, ExpressionNode, Operator, TypeQualifier, UnaryOperator};
use rusty_variant::Variant;
use std::cmp::Ordering;

pub trait ConstLookup {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant>;
}

/// Resolves the value ([Variant]) of a `CONST` expression.
pub trait ConstValueResolver<T, E> {
    /// Resolves the value ([Variant]) of a `CONST` expression.
    fn resolve_const(&self, item: &T) -> Result<Variant, E>;
}

impl<S> ConstValueResolver<CaseInsensitiveString, QError> for S
where
    S: ConstLookup,
{
    fn resolve_const(&self, item: &CaseInsensitiveString) -> Result<Variant, QError> {
        self.get_resolved_constant(item)
            .cloned()
            .ok_or(QError::InvalidConstant)
    }
}

impl<S> ConstValueResolver<Expression, QErrorNode> for S
where
    S: ConstLookup,
{
    fn resolve_const(&self, expression: &Expression) -> Result<Variant, QErrorNode> {
        match expression {
            Expression::SingleLiteral(f) => Ok(Variant::VSingle(*f)),
            Expression::DoubleLiteral(d) => Ok(Variant::VDouble(*d)),
            Expression::StringLiteral(s) => Ok(Variant::VString(s.clone())),
            Expression::IntegerLiteral(i) => Ok(Variant::VInteger(*i)),
            Expression::LongLiteral(l) => Ok(Variant::VLong(*l)),
            Expression::Variable(name_expr, _) => {
                let v = self
                    .resolve_const(name_expr.bare_name())
                    .with_err_no_pos()?;
                if let Some(qualifier) = name_expr.qualifier() {
                    let v_q = match v {
                        Variant::VDouble(_) => TypeQualifier::HashDouble,
                        Variant::VSingle(_) => TypeQualifier::BangSingle,
                        Variant::VInteger(_) => TypeQualifier::PercentInteger,
                        Variant::VLong(_) => TypeQualifier::AmpersandLong,
                        Variant::VString(_) => TypeQualifier::DollarString,
                        _ => {
                            panic!("should not have been possible to store a constant of this type")
                        }
                    };
                    if v_q == qualifier {
                        Ok(v)
                    } else {
                        Err(QError::TypeMismatch).with_err_no_pos()
                    }
                } else {
                    Ok(v)
                }
            }
            Expression::BinaryExpression(op, left, right, _) => {
                let v_left = self.resolve_const(left)?;
                let v_right = self.resolve_const(right)?;
                (match *op {
                    Operator::Less => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| ordering == Ordering::Less)
                        .map(Variant::from),
                    Operator::LessOrEqual => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| ordering == Ordering::Less || ordering == Ordering::Equal)
                        .map(Variant::from),
                    Operator::Equal => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| ordering == Ordering::Equal)
                        .map(Variant::from),
                    Operator::GreaterOrEqual => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| {
                            ordering == Ordering::Greater || ordering == Ordering::Equal
                        })
                        .map(Variant::from),
                    Operator::Greater => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| ordering == Ordering::Greater)
                        .map(Variant::from),
                    Operator::NotEqual => v_left
                        .try_cmp(&v_right)
                        .map(|ordering| ordering == Ordering::Less || ordering == Ordering::Greater)
                        .map(Variant::from),
                    Operator::Plus => v_left.plus(v_right),
                    Operator::Minus => v_left.minus(v_right),
                    Operator::Multiply => v_left.multiply(v_right),
                    Operator::Divide => v_left.divide(v_right),
                    Operator::Modulo => v_left.modulo(v_right),
                    Operator::And => v_left.and(v_right),
                    Operator::Or => v_left.or(v_right),
                })
                .with_err_at(right)
            }
            Expression::UnaryExpression(op, child) => {
                let v = self.resolve_const(child)?;
                (match *op {
                    UnaryOperator::Minus => v.negate(),
                    UnaryOperator::Not => v.unary_not(),
                })
                .with_err_at(child)
            }
            Expression::Parenthesis(child) => self.resolve_const(child),
            Expression::Property(_, _, _)
            | Expression::FunctionCall(_, _)
            | Expression::ArrayElement(_, _, _)
            | Expression::BuiltInFunctionCall(_, _) => {
                Err(QError::InvalidConstant).with_err_no_pos()
            }
        }
    }
}

impl<S> ConstValueResolver<ExpressionNode, QErrorNode> for S
where
    S: ConstLookup,
{
    fn resolve_const(&self, expr_node: &ExpressionNode) -> Result<Variant, QErrorNode> {
        self.resolve_const(&expr_node.element)
            .patch_err_pos(expr_node)
    }
}

impl<S> ConstValueResolver<Box<ExpressionNode>, QErrorNode> for S
where
    S: ConstLookup,
{
    fn resolve_const(&self, item: &Box<ExpressionNode>) -> Result<Variant, QErrorNode> {
        self.resolve_const(item.as_ref())
    }
}
