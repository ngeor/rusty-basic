use crate::common::QError;
use crate::parser::TypeQualifier;
use crate::variant;
use crate::variant::Variant;

// ========================================================
// variant casting
// ========================================================

// https://doc.rust-lang.org/nomicon/casts.html
// 1. casting from an f32 to an f64 is perfect and lossless
// 2. casting from a float to an integer will round the float towards zero
//    NOTE: currently this will cause Undefined Behavior if the rounded value cannot be represented by the target integer type. This includes Inf and NaN. This is a bug and will be fixed.
// 3. casting from an integer to float will produce the floating point representation of the integer, rounded if necessary (rounding to nearest, ties to even)
// 4. casting from an f64 to an f32 will produce the closest possible value (rounding to nearest, ties to even)

pub trait QBNumberCast<T> {
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

impl QBNumberCast<f32> for Variant {
    fn try_cast(&self) -> Result<f32, QError> {
        match self {
            Self::VSingle(f) => Ok(*f),
            Self::VDouble(f) => f.try_cast(),
            Self::VInteger(f) => f.try_cast(),
            Self::VLong(f) => f.try_cast(),
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl QBNumberCast<f64> for Variant {
    fn try_cast(&self) -> Result<f64, QError> {
        match self {
            Self::VSingle(f) => f.try_cast(),
            Self::VDouble(f) => Ok(*f),
            Self::VInteger(f) => f.try_cast(),
            Self::VLong(f) => f.try_cast(),
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl QBNumberCast<i32> for Variant {
    fn try_cast(&self) -> Result<i32, QError> {
        match self {
            Self::VSingle(f) => f.try_cast(),
            Self::VDouble(f) => f.try_cast(),
            Self::VInteger(f) => Ok(*f),
            Self::VLong(f) => f.try_cast(),
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl QBNumberCast<i64> for Variant {
    fn try_cast(&self) -> Result<i64, QError> {
        match self {
            Self::VSingle(f) => f.try_cast(),
            Self::VDouble(f) => f.try_cast(),
            Self::VInteger(f) => f.try_cast(),
            Self::VLong(f) => Ok(*f),
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl Variant {
    pub fn cast(self, target_type: TypeQualifier) -> Result<Self, QError> {
        match target_type {
            TypeQualifier::BangSingle => Ok(Self::VSingle((&self).try_cast()?)),
            TypeQualifier::HashDouble => Ok(Self::VDouble((&self).try_cast()?)),
            TypeQualifier::PercentInteger => Ok(Self::VInteger((&self).try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Self::VLong((&self).try_cast()?)),
            TypeQualifier::DollarString => match self {
                Self::VString(_) => Ok(self),
                _ => Err(QError::TypeMismatch),
            },
        }
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
                Variant::from(1.0_f32)
                    .cast(TypeQualifier::BangSingle)
                    .unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                Variant::from(1.0_f32)
                    .cast(TypeQualifier::HashDouble)
                    .unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            Variant::from(1.0_f32)
                .cast(TypeQualifier::DollarString)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                Variant::from(1.0_f32)
                    .cast(TypeQualifier::PercentInteger)
                    .unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                Variant::from(1.0_f32)
                    .cast(TypeQualifier::AmpersandLong)
                    .unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_double {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                Variant::from(1.0).cast(TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                Variant::from(1.0).cast(TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            Variant::from(1.0)
                .cast(TypeQualifier::DollarString)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                Variant::from(1.0)
                    .cast(TypeQualifier::PercentInteger)
                    .unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                Variant::from(1.0)
                    .cast(TypeQualifier::AmpersandLong)
                    .unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_string {
        use super::*;

        #[test]
        fn to_float() {
            Variant::from("hello")
                .cast(TypeQualifier::BangSingle)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_double() {
            Variant::from("hello")
                .cast(TypeQualifier::HashDouble)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_string() {
            assert_eq!(
                Variant::from("hello")
                    .cast(TypeQualifier::DollarString)
                    .unwrap(),
                Variant::from("hello")
            );
        }

        #[test]
        fn to_integer() {
            Variant::from("hello")
                .cast(TypeQualifier::PercentInteger)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_long() {
            Variant::from("hello")
                .cast(TypeQualifier::AmpersandLong)
                .expect_err("Type mismatch");
        }
    }

    mod from_integer {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                Variant::from(1).cast(TypeQualifier::BangSingle).unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                Variant::from(1).cast(TypeQualifier::HashDouble).unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            Variant::from(1)
                .cast(TypeQualifier::DollarString)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                Variant::from(1)
                    .cast(TypeQualifier::PercentInteger)
                    .unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                Variant::from(1).cast(TypeQualifier::AmpersandLong).unwrap(),
                Variant::from(1_i64)
            );
        }
    }

    mod from_long {
        use super::*;

        #[test]
        fn to_float() {
            assert_eq!(
                Variant::from(1_i64)
                    .cast(TypeQualifier::BangSingle)
                    .unwrap(),
                Variant::from(1.0_f32)
            );
        }

        #[test]
        fn to_double() {
            assert_eq!(
                Variant::from(1_i64)
                    .cast(TypeQualifier::HashDouble)
                    .unwrap(),
                Variant::from(1.0)
            );
        }

        #[test]
        fn to_string() {
            Variant::from(1_i64)
                .cast(TypeQualifier::DollarString)
                .expect_err("Type mismatch");
        }

        #[test]
        fn to_integer() {
            assert_eq!(
                Variant::from(1_i64)
                    .cast(TypeQualifier::PercentInteger)
                    .unwrap(),
                Variant::from(1)
            );
        }

        #[test]
        fn to_long() {
            assert_eq!(
                Variant::from(1_i64)
                    .cast(TypeQualifier::AmpersandLong)
                    .unwrap(),
                Variant::from(1_i64)
            );
        }
    }
}
