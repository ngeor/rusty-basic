use crate::parser::{Operand, TypeQualifier, UnaryOperand};

pub fn cast_binary_op(
    op: Operand,
    left: TypeQualifier,
    right: TypeQualifier,
) -> Option<TypeQualifier> {
    if left == TypeQualifier::FileHandle || right == TypeQualifier::FileHandle {
        // file handles are a special case they're not supposed to mix with others, exit fast
        return None;
    }

    match op {
        // 1. arithmetic operators
        // 1a. plus -> if we can cast left to right, that's the result
        Operand::Plus => {
            if left.can_cast_to(right) {
                Some(left)
            } else {
                None
            }
        }
        // 1b. minus, multiply, divide -> if we can cast left to right, and we're not a string, that's the result
        Operand::Minus | Operand::Multiply | Operand::Divide => {
            if left.can_cast_to(right) && left != TypeQualifier::DollarString {
                Some(left)
            } else {
                None
            }
        }
        // 2. relational operators
        //    if we an cast left to right, the result is -1 or 0, therefore integer
        Operand::Less
        | Operand::LessOrEqual
        | Operand::Equal
        | Operand::GreaterOrEqual
        | Operand::Greater
        | Operand::NotEqual => {
            if left.can_cast_to(right) {
                Some(TypeQualifier::PercentInteger)
            } else {
                None
            }
        }
        // 3. binary operators
        //    they only work if both sides are cast-able to integer, which is also the result type
        Operand::And | Operand::Or => {
            if left.can_cast_to(TypeQualifier::PercentInteger)
                && right.can_cast_to(TypeQualifier::PercentInteger)
            {
                Some(TypeQualifier::PercentInteger)
            } else {
                None
            }
        }
    }
}

pub fn cast_unary_op(_op: UnaryOperand, child: TypeQualifier) -> Option<TypeQualifier> {
    if child == TypeQualifier::FileHandle || child == TypeQualifier::DollarString {
        // file handles are a special case they're not supposed to mix with others,
        // strings don't have any unary operator that can be applied to them
        None
    } else {
        Some(child)
    }
}
