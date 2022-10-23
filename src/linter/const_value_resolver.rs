use crate::common::*;
use crate::parser::{Expression, ExpressionNode, Operator, TypeQualifier, UnaryOperator};
use crate::variant::Variant;
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
                let Locatable { element, pos } = left.as_ref();
                let v_left = self.resolve_const(element).patch_err_pos(*pos)?;
                let Locatable { element, pos } = right.as_ref();
                let v_right = self.resolve_const(element).patch_err_pos(*pos)?;
                match *op {
                    Operator::Less => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less).into())
                    }
                    Operator::LessOrEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less || order == Ordering::Equal).into())
                    }
                    Operator::Equal => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Equal).into())
                    }
                    Operator::GreaterOrEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Greater || order == Ordering::Equal).into())
                    }
                    Operator::Greater => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Greater).into())
                    }
                    Operator::NotEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less || order == Ordering::Greater).into())
                    }
                    Operator::Plus => v_left.plus(v_right).with_err_at(*pos),
                    Operator::Minus => v_left.minus(v_right).with_err_at(*pos),
                    Operator::Multiply => v_left.multiply(v_right).with_err_at(*pos),
                    Operator::Divide => v_left.divide(v_right).with_err_at(*pos),
                    Operator::Modulo => v_left.modulo(v_right).with_err_at(*pos),
                    Operator::And => v_left.and(v_right).with_err_at(*pos),
                    Operator::Or => v_left.or(v_right).with_err_at(*pos),
                }
            }
            Expression::UnaryExpression(op, child) => {
                let Locatable { element, pos } = child.as_ref();
                let v = self.resolve_const(element).patch_err_pos(*pos)?;
                match *op {
                    UnaryOperator::Minus => v.negate().with_err_at(*pos),
                    UnaryOperator::Not => v.unary_not().with_err_at(*pos),
                }
            }
            Expression::Parenthesis(child) => {
                let Locatable { element, pos } = child.as_ref();
                self.resolve_const(element).patch_err_pos(*pos)
            }
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
        self.resolve_const(expr_node.as_ref())
            .patch_err_pos(expr_node)
    }
}
