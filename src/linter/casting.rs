use crate::common::QError;
use crate::parser::{CanCastTo, Operator, TypeQualifier, UnaryOperator};
use crate::variant;
use crate::variant::Variant;

// ========================================================
// binary operator
// ========================================================

pub fn cast_binary_op(
    op: Operator,
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
        Operator::Plus => {
            if left.can_cast_to(right) {
                Some(left)
            } else {
                None
            }
        }
        // 1b. minus, multiply, divide -> if we can cast left to right, and we're not a string, that's the result
        Operator::Minus | Operator::Multiply | Operator::Divide => {
            if left.can_cast_to(right) && left != TypeQualifier::DollarString {
                Some(left)
            } else {
                None
            }
        }
        // 2. relational operators
        //    if we an cast left to right, the result is -1 or 0, therefore integer
        Operator::Less
        | Operator::LessOrEqual
        | Operator::Equal
        | Operator::GreaterOrEqual
        | Operator::Greater
        | Operator::NotEqual => {
            if left.can_cast_to(right) {
                Some(TypeQualifier::PercentInteger)
            } else {
                None
            }
        }
        // 3. binary operators
        //    they only work if both sides are cast-able to integer, which is also the result type
        Operator::And | Operator::Or => {
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

// ========================================================
// unary operator
// ========================================================

pub fn cast_unary_op(_op: UnaryOperator, child: TypeQualifier) -> Option<TypeQualifier> {
    if child == TypeQualifier::FileHandle || child == TypeQualifier::DollarString {
        // file handles are a special case they're not supposed to mix with others,
        // strings don't have any unary operator that can be applied to them
        None
    } else {
        Some(child)
    }
}

// ========================================================
// variant casting
// ========================================================

// https://doc.rust-lang.org/nomicon/casts.html
// 1. casting from an f32 to an f64 is perfect and lossless
// 2. casting from a float to an integer will round the float towards zero
//    NOTE: currently this will cause Undefined Behavior if the rounded value cannot be represented by the target integer type. This includes Inf and NaN. This is a bug and will be fixed.
// 3. casting from an integer to float will produce the floating point representation of the integer, rounded if necessary (rounding to nearest, ties to even)
// 4. casting from an f64 to an f32 will produce the closest possible value (rounding to nearest, ties to even)

trait QBNumberCast<T> {
    fn try_cast(&self) -> Result<T, QError>;
}

impl QBNumberCast<f64> for f32 {
    fn try_cast(&self) -> Result<f64, QError> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i32> for f32 {
    fn try_cast(&self) -> Result<i32, QError> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_INTEGER as f32) && r <= (variant::MAX_INTEGER as f32) {
                Ok(r as i32)
            } else {
                Err(QError::Overflow)
            }
        } else {
            Err(QError::Other(format!("Cannot cast {} to i32", self)))
        }
    }
}

impl QBNumberCast<i64> for f32 {
    fn try_cast(&self) -> Result<i64, QError> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_LONG as f32) && r <= (variant::MAX_LONG as f32) {
                Ok(r as i64)
            } else {
                Err(QError::Overflow)
            }
        } else {
            Err(QError::Other(format!("Cannot cast {} to i64", self)))
        }
    }
}

impl QBNumberCast<f32> for f64 {
    fn try_cast(&self) -> Result<f32, QError> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<i32> for f64 {
    fn try_cast(&self) -> Result<i32, QError> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_INTEGER as f64) && r <= (variant::MAX_INTEGER as f64) {
                Ok(r as i32)
            } else {
                Err(QError::Overflow)
            }
        } else {
            Err(QError::Other(format!("Cannot cast {} to i32", self)))
        }
    }
}

impl QBNumberCast<i64> for f64 {
    fn try_cast(&self) -> Result<i64, QError> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_LONG as f64) && r <= (variant::MAX_LONG as f64) {
                Ok(r as i64)
            } else {
                Err(QError::Overflow)
            }
        } else {
            Err(QError::Other(format!("Cannot cast {} to i64", self)))
        }
    }
}

impl QBNumberCast<f32> for i32 {
    fn try_cast(&self) -> Result<f32, QError> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<f64> for i32 {
    fn try_cast(&self) -> Result<f64, QError> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i64> for i32 {
    fn try_cast(&self) -> Result<i64, QError> {
        Ok(*self as i64)
    }
}

impl QBNumberCast<f32> for i64 {
    fn try_cast(&self) -> Result<f32, QError> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<f64> for i64 {
    fn try_cast(&self) -> Result<f64, QError> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i32> for i64 {
    fn try_cast(&self) -> Result<i32, QError> {
        if *self >= (variant::MIN_INTEGER as i64) && *self <= (variant::MAX_INTEGER as i64) {
            Ok(*self as i32)
        } else {
            Err(QError::Overflow)
        }
    }
}

pub fn cast(value: Variant, target_type: TypeQualifier) -> Result<Variant, QError> {
    match value {
        Variant::VSingle(f) => match target_type {
            TypeQualifier::BangSingle => Ok(value),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
            _ => Err(QError::TypeMismatch),
        },
        Variant::VDouble(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(value),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
            _ => Err(QError::TypeMismatch),
        },
        Variant::VString(_) => match target_type {
            TypeQualifier::DollarString => Ok(value),
            _ => Err(QError::TypeMismatch),
        },
        Variant::VInteger(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::PercentInteger => Ok(value),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
            _ => Err(QError::TypeMismatch),
        },
        Variant::VLong(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(value),
            _ => Err(QError::TypeMismatch),
        },
        Variant::VFileHandle(_) => match target_type {
            TypeQualifier::FileHandle => Ok(value),
            _ => Err(QError::TypeMismatch),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_float {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                cast(Variant::from(1.0_f32), TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                cast(Variant::from(1.0_f32), TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            cast(Variant::from(1.0_f32), TypeQualifier::DollarString).expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                cast(Variant::from(1.0_f32), TypeQualifier::PercentInteger).unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                cast(Variant::from(1.0_f32), TypeQualifier::AmpersandLong).unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_double {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                cast(Variant::from(1.0), TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                cast(Variant::from(1.0), TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            cast(Variant::from(1.0), TypeQualifier::DollarString).expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                cast(Variant::from(1.0), TypeQualifier::PercentInteger).unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                cast(Variant::from(1.0), TypeQualifier::AmpersandLong).unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_string {
        use super::*;

        #[test]
        fn to_float() {
            cast(Variant::from("hello"), TypeQualifier::BangSingle).expect_err("Type mismatch");
        }

        #[test]
        fn to_double() {
            cast(Variant::from("hello"), TypeQualifier::HashDouble).expect_err("Type mismatch");
        }

        #[test]
        fn to_string() {
            assert_eq!(
                cast(Variant::from("hello"), TypeQualifier::DollarString).unwrap(),
                Variant::from("hello")
            );
        }

        #[test]
        fn to_integer() {
            cast(Variant::from("hello"), TypeQualifier::PercentInteger).expect_err("Type mismatch");
        }

        #[test]
        fn to_long() {
            cast(Variant::from("hello"), TypeQualifier::AmpersandLong).expect_err("Type mismatch");
        }
    }

    mod from_integer {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                cast(Variant::from(1), TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                cast(Variant::from(1), TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            cast(Variant::from(1), TypeQualifier::DollarString).expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                cast(Variant::from(1), TypeQualifier::PercentInteger).unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                cast(Variant::from(1), TypeQualifier::AmpersandLong).unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_long {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                cast(Variant::from(1_i64), TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                cast(Variant::from(1_i64), TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            cast(Variant::from(1_i64), TypeQualifier::DollarString).expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                cast(Variant::from(1_i64), TypeQualifier::PercentInteger).unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                cast(Variant::from(1_i64), TypeQualifier::AmpersandLong).unwrap(),
                Variant::from(1_i64)
            );
        }
    }
}
