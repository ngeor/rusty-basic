use rusty_common::AtPos;
use rusty_parser::{
    Expression, ExpressionPos, ExpressionType, HasExpressionType, Operator, TypeQualifier
};

use crate::core::{CanCastTo, LintError, LintErrorPos};

pub fn binary_cast(
    left: ExpressionPos,
    right: ExpressionPos,
    op: Operator,
) -> Result<Expression, LintErrorPos> {
    // get the types
    let t_left = left.expression_type();
    let t_right = right.expression_type();
    // get the cast type
    match cast_binary_op_et(&t_left, &t_right, op) {
        Some(type_definition) => Ok(Expression::BinaryExpression(
            op,
            Box::new(left),
            Box::new(right),
            type_definition,
        )),
        None => Err(LintError::TypeMismatch.at(&right)),
    }
}

fn bigger_numeric_type(left: TypeQualifier, other: TypeQualifier) -> Option<TypeQualifier> {
    match left {
        TypeQualifier::BangSingle => match other {
            TypeQualifier::BangSingle
            | TypeQualifier::PercentInteger
            | TypeQualifier::AmpersandLong => Some(TypeQualifier::BangSingle),
            TypeQualifier::HashDouble => Some(TypeQualifier::HashDouble),
            _ => None,
        },
        TypeQualifier::HashDouble => match other {
            TypeQualifier::BangSingle
            | TypeQualifier::PercentInteger
            | TypeQualifier::AmpersandLong
            | TypeQualifier::HashDouble => Some(TypeQualifier::HashDouble),
            _ => None,
        },
        TypeQualifier::PercentInteger => match other {
            TypeQualifier::BangSingle => Some(TypeQualifier::BangSingle),
            TypeQualifier::HashDouble => Some(TypeQualifier::HashDouble),
            TypeQualifier::PercentInteger => Some(TypeQualifier::PercentInteger),
            TypeQualifier::AmpersandLong => Some(TypeQualifier::AmpersandLong),
            _ => None,
        },
        TypeQualifier::AmpersandLong => match other {
            TypeQualifier::BangSingle => Some(TypeQualifier::BangSingle),
            TypeQualifier::HashDouble => Some(TypeQualifier::HashDouble),
            TypeQualifier::PercentInteger | TypeQualifier::AmpersandLong => {
                Some(TypeQualifier::AmpersandLong)
            }
            _ => None,
        },
        _ => None,
    }
}

fn cast_binary_op_q(
    left: TypeQualifier,
    right: TypeQualifier,
    op: Operator,
) -> Option<TypeQualifier> {
    match op {
        // 1. arithmetic operators
        // 1a. plus -> if we can cast self to right, that's the result
        Operator::Plus => {
            match bigger_numeric_type(left, right) {
                Some(result) => Some(result),
                None => {
                    if left == TypeQualifier::DollarString && right == TypeQualifier::DollarString {
                        // string concatenation
                        Some(left)
                    } else {
                        None
                    }
                }
            }
        }
        // 1b. minus, multiply, divide -> if we can cast self to right, and we're not a string, that's the result
        // MOD is covered later in logical operators because it's similar logic
        Operator::Minus | Operator::Multiply | Operator::Divide => bigger_numeric_type(left, right),
        // 2. relational operators
        //    if we an cast self to right, the result is -1 or 0, therefore integer
        Operator::Less
        | Operator::LessOrEqual
        | Operator::Equal
        | Operator::GreaterOrEqual
        | Operator::Greater
        | Operator::NotEqual => {
            if left.can_cast_to(&right) {
                Some(TypeQualifier::PercentInteger)
            } else {
                None
            }
        }
        // 3. logical operators, modulo operator
        //    they only work if both sides are cast-able to integer, which is also the result type
        Operator::And | Operator::Or | Operator::Modulo => {
            if left.can_cast_to(&TypeQualifier::PercentInteger)
                && right.can_cast_to(&TypeQualifier::PercentInteger)
            {
                Some(TypeQualifier::PercentInteger)
            } else {
                None
            }
        }
    }
}

fn cast_binary_op_et(
    left: &ExpressionType,
    right: &ExpressionType,
    op: Operator,
) -> Option<ExpressionType> {
    match left {
        ExpressionType::BuiltIn(q_left) => match right {
            ExpressionType::BuiltIn(q_right) => {
                cast_binary_op_q(*q_left, *q_right, op).map(ExpressionType::BuiltIn)
            }
            ExpressionType::FixedLengthString(_) => {
                cast_binary_op_q(*q_left, TypeQualifier::DollarString, op)
                    .map(ExpressionType::BuiltIn)
            }
            _ => None,
        },
        ExpressionType::FixedLengthString(_) => match right {
            ExpressionType::BuiltIn(TypeQualifier::DollarString)
            | ExpressionType::FixedLengthString(_) => {
                cast_binary_op_q(TypeQualifier::DollarString, TypeQualifier::DollarString, op)
                    .map(ExpressionType::BuiltIn)
            }
            _ => None,
        },
        _ => None,
    }
}
