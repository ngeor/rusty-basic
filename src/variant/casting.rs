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

impl Variant {
    pub fn cast(self, target_type: TypeQualifier) -> Result<Self, QError> {
        match self {
            Self::VSingle(f) => match target_type {
                TypeQualifier::BangSingle => Ok(self),
                TypeQualifier::HashDouble => Ok(Self::VDouble(f.try_cast()?)),
                TypeQualifier::PercentInteger => Ok(Self::VInteger(f.try_cast()?)),
                TypeQualifier::AmpersandLong => Ok(Self::VLong(f.try_cast()?)),
                _ => Err(QError::TypeMismatch),
            },
            Self::VDouble(f) => match target_type {
                TypeQualifier::BangSingle => Ok(Self::VSingle(f.try_cast()?)),
                TypeQualifier::HashDouble => Ok(self),
                TypeQualifier::PercentInteger => Ok(Self::VInteger(f.try_cast()?)),
                TypeQualifier::AmpersandLong => Ok(Self::VLong(f.try_cast()?)),
                _ => Err(QError::TypeMismatch),
            },
            Self::VString(_) => match target_type {
                TypeQualifier::DollarString => Ok(self),
                _ => Err(QError::TypeMismatch),
            },
            Self::VInteger(f) => match target_type {
                TypeQualifier::BangSingle => Ok(Self::VSingle(f.try_cast()?)),
                TypeQualifier::HashDouble => Ok(Self::VDouble(f.try_cast()?)),
                TypeQualifier::PercentInteger => Ok(self),
                TypeQualifier::AmpersandLong => Ok(Self::VLong(f.try_cast()?)),
                _ => Err(QError::TypeMismatch),
            },
            Self::VLong(f) => match target_type {
                TypeQualifier::BangSingle => Ok(Self::VSingle(f.try_cast()?)),
                TypeQualifier::HashDouble => Ok(Self::VDouble(f.try_cast()?)),
                TypeQualifier::PercentInteger => Ok(Self::VInteger(f.try_cast()?)),
                TypeQualifier::AmpersandLong => Ok(self),
                _ => Err(QError::TypeMismatch),
            },
            Self::VFileHandle(_) | Self::VUserDefined(_) => Err(QError::TypeMismatch),
        }
    }
}

// TODO fix all unimplemented
// TODO fix all panic
// TODO fix all unwrap
// TODO fix all try_unwrap
// TODO remove all std::rc and std::cell

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
