use crate::common::*;
use crate::parser::{Expression, ExpressionNode, Name, Operator, TypeQualifier, UnaryOperator};
use crate::variant::Variant;
use std::cmp::Ordering;

pub trait ConstValueResolver {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant>;

    fn resolve_const_value_node(&self, expr_node: &ExpressionNode) -> Result<Variant, QErrorNode> {
        self.resolve_const_value(expr_node.as_ref())
            .patch_err_pos(expr_node)
    }

    fn resolve_const_value(&self, expression: &Expression) -> Result<Variant, QErrorNode> {
        match expression {
            Expression::SingleLiteral(f) => Ok(Variant::VSingle(*f)),
            Expression::DoubleLiteral(d) => Ok(Variant::VDouble(*d)),
            Expression::StringLiteral(s) => Ok(Variant::VString(s.clone())),
            Expression::IntegerLiteral(i) => Ok(Variant::VInteger(*i)),
            Expression::LongLiteral(l) => Ok(Variant::VLong(*l)),
            Expression::VariableName(name) => match name {
                Name::Bare(name) => match self.get_resolved_constant(name) {
                    Some(v) => Ok(v.clone()),
                    None => Err(QError::InvalidConstant).with_err_no_pos(),
                },
                Name::Qualified { bare_name: name, qualifier } => match self.get_resolved_constant(name) {
                    Some(v) => {
                        let v_q = match v {
                            Variant::VDouble(_) => TypeQualifier::HashDouble,
                            Variant::VSingle(_) => TypeQualifier::BangSingle,
                            Variant::VInteger(_) => TypeQualifier::PercentInteger,
                            Variant::VLong(_) => TypeQualifier::AmpersandLong,
                            Variant::VString(_) => TypeQualifier::DollarString,
                            _ => panic!(
                                "should not have been possible to store a constant of this type"
                            ),
                        };
                        if v_q == *qualifier {
                            Ok(v.clone())
                        } else {
                            Err(QError::TypeMismatch).with_err_no_pos()
                        }
                    }
                    None => Err(QError::InvalidConstant).with_err_no_pos(),
                },
            },
            Expression::FunctionCall(_, _) => Err(QError::InvalidConstant).with_err_no_pos(),
            Expression::BinaryExpression(op, left, right) => {
                let Locatable { element, pos } = left.as_ref();
                let v_left = self.resolve_const_value(element).patch_err_pos(*pos)?;
                let Locatable { element, pos } = right.as_ref();
                let v_right = self.resolve_const_value(element).patch_err_pos(*pos)?;
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
                    Operator::And => v_left.and(v_right).with_err_at(*pos),
                    Operator::Or => v_left.or(v_right).with_err_at(*pos),
                }
            }
            Expression::UnaryExpression(op, child) => {
                let Locatable { element, pos } = child.as_ref();
                let v = self.resolve_const_value(element).patch_err_pos(*pos)?;
                match *op {
                    UnaryOperator::Minus => v.negate().with_err_at(*pos),
                    UnaryOperator::Not => v.unary_not().with_err_at(*pos),
                }
            }
            Expression::Parenthesis(child) => {
                let Locatable { element, pos } = child.as_ref();
                self.resolve_const_value(element).patch_err_pos(*pos)
            }
            Expression::FileHandle(_) => Err(QError::InvalidConstant).with_err_no_pos(),
        }
    }
}
