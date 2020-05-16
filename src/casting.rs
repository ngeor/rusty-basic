use crate::parser::TypeQualifier;
use crate::variant;
use crate::variant::Variant;

// https://doc.rust-lang.org/nomicon/casts.html
// 1. casting from an f32 to an f64 is perfect and lossless
// 2. casting from a float to an integer will round the float towards zero
//    NOTE: currently this will cause Undefined Behavior if the rounded value cannot be represented by the target integer type. This includes Inf and NaN. This is a bug and will be fixed.
// 3. casting from an integer to float will produce the floating point representation of the integer, rounded if necessary (rounding to nearest, ties to even)
// 4. casting from an f64 to an f32 will produce the closest possible value (rounding to nearest, ties to even)

trait QBNumberCast<T> {
    fn try_cast(&self) -> Result<T, String>;
}

impl QBNumberCast<f64> for f32 {
    fn try_cast(&self) -> Result<f64, String> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i32> for f32 {
    fn try_cast(&self) -> Result<i32, String> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_INTEGER as f32) && r <= (variant::MAX_INTEGER as f32) {
                Ok(r as i32)
            } else {
                Err("Overflow".to_string())
            }
        } else {
            Err(format!("Cannot cast {} to i32", self))
        }
    }
}

impl QBNumberCast<i64> for f32 {
    fn try_cast(&self) -> Result<i64, String> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_LONG as f32) && r <= (variant::MAX_LONG as f32) {
                Ok(r as i64)
            } else {
                Err("Overflow".to_string())
            }
        } else {
            Err(format!("Cannot cast {} to i64", self))
        }
    }
}

impl QBNumberCast<f32> for f64 {
    fn try_cast(&self) -> Result<f32, String> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<i32> for f64 {
    fn try_cast(&self) -> Result<i32, String> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_INTEGER as f64) && r <= (variant::MAX_INTEGER as f64) {
                Ok(r as i32)
            } else {
                Err("Overflow".to_string())
            }
        } else {
            Err(format!("Cannot cast {} to i32", self))
        }
    }
}

impl QBNumberCast<i64> for f64 {
    fn try_cast(&self) -> Result<i64, String> {
        if self.is_finite() {
            let r = self.round();
            if r >= (variant::MIN_LONG as f64) && r <= (variant::MAX_LONG as f64) {
                Ok(r as i64)
            } else {
                Err("Overflow".to_string())
            }
        } else {
            Err(format!("Cannot cast {} to i64", self))
        }
    }
}

impl QBNumberCast<f32> for i32 {
    fn try_cast(&self) -> Result<f32, String> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<f64> for i32 {
    fn try_cast(&self) -> Result<f64, String> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i64> for i32 {
    fn try_cast(&self) -> Result<i64, String> {
        Ok(*self as i64)
    }
}

impl QBNumberCast<f32> for i64 {
    fn try_cast(&self) -> Result<f32, String> {
        Ok(*self as f32)
    }
}

impl QBNumberCast<f64> for i64 {
    fn try_cast(&self) -> Result<f64, String> {
        Ok(*self as f64)
    }
}

impl QBNumberCast<i32> for i64 {
    fn try_cast(&self) -> Result<i32, String> {
        if *self >= (variant::MIN_INTEGER as i64) && *self <= (variant::MAX_INTEGER as i64) {
            Ok(*self as i32)
        } else {
            Err("Overflow".to_string())
        }
    }
}

pub fn cast(value: Variant, target_type: TypeQualifier) -> Result<Variant, String> {
    match value {
        Variant::VSingle(f) => match target_type {
            TypeQualifier::BangSingle => Ok(value),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VDouble(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(value),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VString(_) => match target_type {
            TypeQualifier::DollarString => Ok(value),
            _ => Err("Type mismatch".to_string()),
        },
        Variant::VInteger(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(value),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VLong(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(value),
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
