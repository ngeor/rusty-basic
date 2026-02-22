use std::cmp::Ordering;
use std::fmt::Display;

use rusty_bit_vec::{MIN_INTEGER, MIN_LONG};

use crate::fit::FitToType;
use crate::{UserDefinedTypeValue, VArray, qb_and, qb_or};

#[derive(Clone, Debug)]
pub enum Variant {
    VSingle(f32),
    VDouble(f64),
    VString(String),
    VInteger(i32),
    VLong(i64),
    VUserDefined(Box<UserDefinedTypeValue>),
    VArray(Box<VArray>),
}

#[derive(Debug)]
pub enum VariantError {
    DivisionByZero,
    Overflow,
    TypeMismatch,
}

pub const V_TRUE: Variant = Variant::VInteger(-1);
pub const V_FALSE: Variant = Variant::VInteger(0);

trait ApproximateCmp {
    fn cmp(left: &Self, right: &Self) -> Ordering;
}

impl ApproximateCmp for f32 {
    fn cmp(left: &Self, right: &Self) -> Ordering {
        let diff = left - right;
        if diff < -0.00001 {
            Ordering::Less
        } else if diff > 0.00001 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl ApproximateCmp for f64 {
    fn cmp(left: &Self, right: &Self) -> Ordering {
        let diff = left - right;
        if diff < -0.00001 {
            Ordering::Less
        } else if diff > 0.00001 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

trait ApproximateEqToInt {
    fn approximate_eq(self, right: i32) -> bool;
}

impl ApproximateEqToInt for i32 {
    fn approximate_eq(self, right: i32) -> bool {
        self == right
    }
}

impl ApproximateEqToInt for i64 {
    fn approximate_eq(self, right: i32) -> bool {
        self == right.into()
    }
}

impl ApproximateEqToInt for f32 {
    fn approximate_eq(self, right: i32) -> bool {
        (self - right as Self).abs() < 0.00001
    }
}

impl ApproximateEqToInt for f64 {
    fn approximate_eq(self, right: i32) -> bool {
        (self - right as Self).abs() < 0.00001
    }
}

macro_rules! div {
    ($nom:expr, $div:expr) => {
        if $div.approximate_eq(0) {
            Err($crate::VariantError::DivisionByZero)
        } else {
            Ok(($nom / $div).fit_to_type())
        }
    };

    ($nom:expr, $div:expr, $cast:tt) => {
        if $div.approximate_eq(0) {
            Err($crate::VariantError::DivisionByZero)
        } else {
            Ok(($nom as $cast / $div as $cast).fit_to_type())
        }
    };
}

// TODO implement standard operators with panics, let the linter guarantee the type compatibility

impl Variant {
    pub fn try_cmp(&self, other: &Self) -> Result<Ordering, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => Ok(ApproximateCmp::cmp(f_left, f_right)),
                Self::VDouble(d_right) => Ok(ApproximateCmp::cmp(&(*f_left as f64), d_right)),
                Self::VInteger(i_right) => Ok(ApproximateCmp::cmp(f_left, &(*i_right as f32))),
                Self::VLong(l_right) => Ok(ApproximateCmp::cmp(f_left, &(*l_right as f32))),
                _ => other.try_cmp(self).map(|x| x.reverse()),
            },
            Self::VDouble(d_left) => match other {
                Self::VDouble(d_right) => Ok(ApproximateCmp::cmp(d_left, d_right)),
                Self::VInteger(i_right) => Ok(ApproximateCmp::cmp(d_left, &(*i_right as f64))),
                Self::VLong(l_right) => Ok(ApproximateCmp::cmp(d_left, &(*l_right as f64))),
                _ => other.try_cmp(self).map(|x| x.reverse()),
            },
            Self::VString(s_left) => match other {
                Self::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VInteger(i_left) => match other {
                Self::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                Self::VLong(l_right) => Ok((*i_left as i64).cmp(l_right)),
                _ => other.try_cmp(self).map(|x| x.reverse()),
            },
            Self::VLong(l_left) => match other {
                Self::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => other.try_cmp(self).map(|x| x.reverse()),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    fn cmp_same_type_only(&self, other: &Self) -> Result<Ordering, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => Ok(ApproximateCmp::cmp(f_left, f_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VDouble(d_left) => match other {
                Self::VDouble(d_right) => Ok(ApproximateCmp::cmp(d_left, d_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VString(s_left) => match other {
                Self::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VInteger(i_left) => match other {
                Self::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VLong(l_left) => match other {
                Self::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => Err(VariantError::TypeMismatch),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn negate(self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(n) => Ok(Self::VSingle(-n)),
            Self::VDouble(n) => Ok(Self::VDouble(-n)),
            Self::VInteger(n) => {
                if n <= MIN_INTEGER {
                    // prevent converting -32768 to 32768
                    Err(VariantError::Overflow)
                } else {
                    Ok(Self::VInteger(-n))
                }
            }
            Self::VLong(n) => {
                if n <= MIN_LONG {
                    Err(VariantError::Overflow)
                } else {
                    Ok(Self::VLong(-n))
                }
            }
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn unary_not(self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f) => Ok(Self::VSingle(-f.round() - 1.0)),
            Self::VDouble(d) => Ok(Self::VDouble(-d.round() - 1.0)),
            Self::VInteger(n) => Ok(Self::VInteger(-n - 1)),
            Self::VLong(n) => Ok(Self::VLong(-n - 1)),
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn plus(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => Ok(Self::VSingle(f_left + f_right)),
                Self::VDouble(d_right) => Ok(Self::VDouble(f_left as f64 + d_right)),
                Self::VInteger(i_right) => Ok(Self::VSingle(f_left + i_right as f32)),
                Self::VLong(l_right) => Ok(Self::VSingle(f_left + l_right as f32)),
                _ => other.plus(self),
            },
            Self::VDouble(d_left) => match other {
                Self::VDouble(d_right) => Ok(Self::VDouble(d_left + d_right)),
                Self::VInteger(i_right) => Ok(Self::VDouble(d_left + i_right as f64)),
                Self::VLong(l_right) => Ok(Self::VDouble(d_left + l_right as f64)),
                _ => other.plus(self),
            },
            Self::VString(s_left) => match other {
                Self::VString(s_right) => Ok(Self::VString(format!("{}{}", s_left, s_right))),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VInteger(i_left) => match other {
                Self::VInteger(i_right) => Ok(Self::VInteger(i_left + i_right)),
                Self::VLong(l_right) => Ok(Self::VLong(i_left as i64 + l_right)),
                _ => other.plus(self),
            },
            Self::VLong(l_left) => match other {
                Self::VLong(l_right) => Ok(Self::VLong(l_left + l_right)),
                _ => other.plus(self),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn minus(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => Ok(Self::VSingle(f_left - f_right)),
                Self::VDouble(d_right) => Ok(Self::VDouble(f_left as f64 - d_right)),
                Self::VInteger(i_right) => Ok(Self::VSingle(f_left - i_right as f32)),
                Self::VLong(l_right) => Ok(Self::VSingle(f_left - l_right as f32)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Self::VDouble(d_left) => match other {
                Self::VDouble(d_right) => Ok(Self::VDouble(d_left - d_right)),
                Self::VInteger(i_right) => Ok(Self::VDouble(d_left - i_right as f64)),
                Self::VLong(l_right) => Ok(Self::VDouble(d_left - l_right as f64)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Self::VInteger(i_left) => match other {
                Self::VInteger(i_right) => Ok(Self::VInteger(i_left - i_right)),
                Self::VLong(l_right) => Ok(Self::VLong(i_left as i64 - l_right)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Self::VLong(l_left) => match other {
                Self::VLong(l_right) => Ok(Self::VLong(l_left - l_right)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn multiply(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => Ok(Self::VSingle(f_left * f_right)),
                Self::VDouble(d_right) => Ok(Self::VDouble(f_left as f64 * d_right)),
                Self::VInteger(i_right) => Ok(Self::VSingle(f_left * i_right as f32)),
                Self::VLong(l_right) => Ok(Self::VSingle(f_left * l_right as f32)),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VDouble(d_left) => match other {
                Self::VDouble(d_right) => Ok(Self::VDouble(d_left * d_right)),
                Self::VInteger(i_right) => Ok(Self::VDouble(d_left * i_right as f64)),
                Self::VLong(l_right) => Ok(Self::VDouble(d_left * l_right as f64)),
                _ => other.multiply(self),
            },
            Self::VInteger(i_left) => match other {
                Self::VInteger(i_right) => Ok(Self::VInteger(i_left * i_right)),
                Self::VLong(l_right) => Ok(Self::VLong(i_left as i64 * l_right)),
                _ => other.multiply(self),
            },
            Self::VLong(l_left) => match other {
                Self::VLong(l_right) => Ok(Self::VLong(l_left * l_right)),
                _ => other.multiply(self),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn divide(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f_left) => match other {
                Self::VSingle(f_right) => div!(f_left, f_right),
                Self::VDouble(d_right) => div!(f_left, d_right, f64),
                Self::VInteger(i_right) => div!(f_left, i_right, f32),
                Self::VLong(l_right) => div!(f_left, l_right, f32),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VDouble(d_left) => match other {
                Self::VSingle(f_right) => div!(d_left, f_right, f64),
                Self::VDouble(d_right) => div!(d_left, d_right),
                Self::VInteger(i_right) => div!(d_left, i_right, f64),
                Self::VLong(l_right) => div!(d_left, l_right, f64),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VInteger(i_left) => match other {
                Self::VSingle(f_right) => div!(i_left, f_right, f32),
                Self::VDouble(d_right) => div!(i_left, d_right, f64),
                Self::VInteger(i_right) => div!(i_left, i_right, f32),
                Self::VLong(l_right) => div!(i_left, l_right, f32),
                _ => Err(VariantError::TypeMismatch),
            },
            Self::VLong(l_left) => match other {
                Self::VSingle(f_right) => div!(l_left, f_right, f32),
                Self::VDouble(d_right) => div!(l_left, d_right, f64),
                Self::VInteger(i_right) => div!(l_left, i_right, f32),
                Self::VLong(l_right) => div!(l_left, l_right, f32),
                _ => Err(VariantError::TypeMismatch),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn modulo(self, other: Self) -> Result<Self, VariantError> {
        let round_left = self.round()?;
        let round_right = other.round()?;
        if round_right.is_approximately_zero()? {
            Err(VariantError::DivisionByZero)
        } else {
            match round_left {
                Self::VInteger(i_left) => match round_right {
                    Self::VInteger(i_right) => Ok(Self::VInteger(i_left % i_right)),
                    Self::VLong(_) => Err(VariantError::Overflow),
                    _ => Err(VariantError::TypeMismatch),
                },
                Self::VLong(_) => Err(VariantError::Overflow),
                _ => Err(VariantError::TypeMismatch),
            }
        }
    }

    fn round(self) -> Result<Self, VariantError> {
        match self {
            Self::VSingle(f) => Ok(f.round().fit_to_type()),
            Self::VDouble(d) => Ok(d.round().fit_to_type()),
            Self::VInteger(_) | Self::VLong(_) => Ok(self),
            _ => Err(VariantError::TypeMismatch),
        }
    }

    fn is_approximately_zero(&self) -> Result<bool, VariantError> {
        match self {
            Self::VSingle(f) => Ok((*f).approximate_eq(0)),
            Self::VDouble(d) => Ok((*d).approximate_eq(0)),
            Self::VInteger(i) => Ok(*i == 0),
            Self::VLong(l) => Ok(*l == 0),
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn and(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VInteger(a) => match other {
                Self::VInteger(b) => Ok(Self::VInteger(qb_and(a, b))),
                _ => Err(VariantError::TypeMismatch),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }

    pub fn or(self, other: Self) -> Result<Self, VariantError> {
        match self {
            Self::VInteger(a) => match other {
                Self::VInteger(b) => Ok(Self::VInteger(qb_or(a, b))),
                _ => Err(VariantError::TypeMismatch),
            },
            _ => Err(VariantError::TypeMismatch),
        }
    }
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        match self.cmp_same_type_only(other) {
            Ok(ord) => ord == Ordering::Equal,
            _ => false,
        }
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VSingle(n) => write!(f, "{}", n),
            Self::VDouble(n) => write!(f, "{}", n),
            Self::VString(s) => write!(f, "{}", s),
            Self::VInteger(n) => write!(f, "{}", n),
            Self::VLong(n) => write!(f, "{}", n),
            _ => Err(std::fmt::Error),
        }
    }
}

// ========================================================
// Convert from standard types to Variant
// ========================================================

impl From<f32> for Variant {
    fn from(f: f32) -> Self {
        Self::VSingle(f)
    }
}

impl From<f64> for Variant {
    fn from(f: f64) -> Self {
        Self::VDouble(f)
    }
}

impl From<String> for Variant {
    fn from(s: String) -> Self {
        Self::VString(s)
    }
}

impl From<&str> for Variant {
    fn from(s: &str) -> Self {
        Self::VString(s.to_string())
    }
}

impl From<i32> for Variant {
    fn from(i: i32) -> Self {
        Self::VInteger(i)
    }
}

impl From<i64> for Variant {
    fn from(i: i64) -> Self {
        Self::VLong(i)
    }
}

impl From<bool> for Variant {
    fn from(b: bool) -> Self {
        if b { V_TRUE } else { V_FALSE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fmt {
        use super::*;

        #[test]
        fn test_fmt() {
            assert_eq!(Variant::VSingle(1.1).to_string(), "1.1");
            assert_eq!(Variant::VDouble(1.1).to_string(), "1.1");
            assert_eq!(
                Variant::VString("hello, world".to_string()).to_string(),
                "hello, world"
            );
            assert_eq!(Variant::VInteger(42).to_string(), "42");
            assert_eq!(Variant::VLong(42).to_string(), "42");
        }
    }

    mod from {
        use super::*;

        #[test]
        fn test_from() {
            assert_eq!(Variant::from(3.14_f32), Variant::VSingle(3.14));
            assert_eq!(Variant::from(3.14), Variant::VDouble(3.14));
            assert_eq!(
                Variant::from("hello"),
                Variant::VString("hello".to_string())
            );
            assert_eq!(Variant::from(42), Variant::VInteger(42));
            assert_eq!(Variant::from(42_i64), Variant::VLong(42));
            assert_eq!(Variant::from(true), V_TRUE);
            assert_eq!(Variant::from(false), V_FALSE);
        }
    }

    mod plus {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VSingle(1.1).plus(Variant::VSingle(2.4)).unwrap(),
                    Variant::VSingle(3.5)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VSingle(1.1).plus(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .plus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VSingle(1.1).plus(Variant::VInteger(2)).unwrap(),
                    Variant::VSingle(3.1)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VSingle(1.1).plus(Variant::VLong(2)).unwrap(),
                    Variant::VSingle(3.1)
                );
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VDouble(1.1).plus(Variant::VSingle(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VDouble(1.1).plus(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .plus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VDouble(1.1).plus(Variant::VInteger(2)).unwrap(),
                    Variant::VDouble(3.1)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VDouble(1.1).plus(Variant::VLong(2)).unwrap(),
                    Variant::VDouble(3.1)
                );
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .plus(Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .plus(Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                assert_eq!(
                    Variant::VString("hello".to_string())
                        .plus(Variant::VString(" world".to_string()))
                        .unwrap(),
                    Variant::VString("hello world".to_string())
                );
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .plus(Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .plus(Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(1).plus(Variant::VSingle(0.5)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 1.5),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VInteger(1).plus(Variant::VDouble(0.6)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 1.6),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .plus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VInteger(42).plus(Variant::VInteger(2)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VInteger(42).plus(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod long {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(1).plus(Variant::VSingle(2.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(1).plus(Variant::VDouble(2.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .plus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VLong(42).plus(Variant::VInteger(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VLong(42).plus(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }
        }
    }

    mod minus {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VSingle(5.9).minus(Variant::VSingle(2.4)).unwrap(),
                    Variant::VSingle(3.5)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VSingle(5.9).minus(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .minus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VSingle(5.1).minus(Variant::VInteger(2)).unwrap(),
                    Variant::VSingle(3.1)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VSingle(5.1).minus(Variant::VLong(2)).unwrap(),
                    Variant::VSingle(3.1)
                );
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VDouble(5.9).minus(Variant::VSingle(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VDouble(5.9).minus(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(3.5)
                );
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .minus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VDouble(5.1).minus(Variant::VInteger(2)).unwrap(),
                    Variant::VDouble(3.1)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VDouble(5.1).minus(Variant::VLong(2)).unwrap(),
                    Variant::VDouble(3.1)
                );
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .minus(Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .minus(Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                Variant::VString("hello".to_string())
                    .minus(Variant::VString("world".to_string()))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .minus(Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .minus(Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(31).minus(Variant::VSingle(13.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 18.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VInteger(31).minus(Variant::VDouble(13.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 18.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .minus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VInteger(42).minus(Variant::VInteger(2)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VInteger(42).minus(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod long {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(5).minus(Variant::VSingle(2.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(5).minus(Variant::VDouble(2.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .minus("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VLong(42).minus(Variant::VInteger(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VLong(42).minus(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }
        }
    }

    mod multiply {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VSingle(5.9)
                        .multiply(Variant::VSingle(2.4))
                        .unwrap(),
                    Variant::VSingle(14.16)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VSingle(5.9)
                        .multiply(Variant::VDouble(2.4))
                        .unwrap(),
                    Variant::VDouble(14.16)
                );
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .multiply("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VSingle(5.1)
                        .multiply(Variant::VInteger(2))
                        .unwrap(),
                    Variant::VSingle(10.2)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VSingle(5.1).multiply(Variant::VLong(2)).unwrap(),
                    Variant::VSingle(10.2)
                );
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VDouble(5.9)
                        .multiply(Variant::VSingle(2.4))
                        .unwrap(),
                    Variant::VDouble(14.16)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VDouble(5.9)
                        .multiply(Variant::VDouble(2.4))
                        .unwrap(),
                    Variant::VDouble(14.16)
                );
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .multiply("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VDouble(5.1)
                        .multiply(Variant::VInteger(2))
                        .unwrap(),
                    Variant::VDouble(10.2)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VDouble(5.1).multiply(Variant::VLong(2)).unwrap(),
                    Variant::VDouble(10.2)
                );
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .multiply(Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .multiply(Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                Variant::VString("hello".to_string())
                    .multiply(Variant::VString("world".to_string()))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .multiply(Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .multiply(Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(31)
                    .multiply(Variant::VSingle(13.0))
                    .unwrap()
                {
                    Variant::VSingle(result) => assert_eq!(result, 403.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VInteger(31)
                    .multiply(Variant::VDouble(13.0))
                    .unwrap()
                {
                    Variant::VDouble(result) => assert_eq!(result, 403.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .multiply("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VInteger(42)
                    .multiply(Variant::VInteger(2))
                    .unwrap()
                {
                    Variant::VInteger(result) => assert_eq!(result, 84),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VInteger(42).multiply(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 84),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod long {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(5).multiply(Variant::VSingle(2.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 10.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(5).multiply(Variant::VDouble(2.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 10.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .multiply("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VLong(42).multiply(Variant::VInteger(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 84),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VLong(42).multiply(Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 84),
                    _ => panic!("assertion failed"),
                }
            }
        }
    }

    mod divide {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VSingle(5.0).divide(Variant::VSingle(2.0)).unwrap(),
                    Variant::VSingle(2.5)
                );
            }

            #[test]
            fn test_single_no_fraction() {
                assert_eq!(
                    Variant::VSingle(4.0).divide(Variant::VSingle(2.0)).unwrap(),
                    Variant::VInteger(2)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VSingle(5.9).divide(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(2.45833333333333)
                );
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .divide("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VSingle(5.1).divide(Variant::VInteger(2)).unwrap(),
                    Variant::VSingle(2.55)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VSingle(5.1).divide(Variant::VLong(2)).unwrap(),
                    Variant::VSingle(2.55)
                );
            }

            #[test]
            fn test_division_by_zero() {
                Variant::VSingle(1.0)
                    .divide(Variant::VSingle(0.0))
                    .expect_err("Division by zero");
                Variant::VSingle(1.0)
                    .divide(Variant::VDouble(0.0))
                    .expect_err("Division by zero");
                Variant::VSingle(1.0)
                    .divide(Variant::VInteger(0))
                    .expect_err("Division by zero");
                Variant::VSingle(1.0)
                    .divide(Variant::VLong(0))
                    .expect_err("Division by zero");
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                assert_eq!(
                    Variant::VDouble(5.9).divide(Variant::VSingle(2.4)).unwrap(),
                    Variant::VDouble(2.45833333333333)
                );
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VDouble(5.9).divide(Variant::VDouble(2.4)).unwrap(),
                    Variant::VDouble(2.45833333333333)
                );
            }

            #[test]
            fn test_double_no_fraction() {
                assert_eq!(
                    Variant::VDouble(21.0)
                        .divide(Variant::VDouble(3.0))
                        .unwrap(),
                    Variant::VInteger(7)
                );
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .divide("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VDouble(5.1).divide(Variant::VInteger(2)).unwrap(),
                    Variant::VDouble(2.55)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VDouble(5.1).divide(Variant::VLong(2)).unwrap(),
                    Variant::VDouble(2.55)
                );
            }

            #[test]
            fn test_division_by_zero() {
                Variant::VDouble(1.0)
                    .divide(Variant::VSingle(0.0))
                    .expect_err("Division by zero");
                Variant::VDouble(1.0)
                    .divide(Variant::VDouble(0.0))
                    .expect_err("Division by zero");
                Variant::VDouble(1.0)
                    .divide(Variant::VInteger(0))
                    .expect_err("Division by zero");
                Variant::VDouble(1.0)
                    .divide(Variant::VLong(0))
                    .expect_err("Division by zero");
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .divide(Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .divide(Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                Variant::VString("hello".to_string())
                    .divide(Variant::VString("world".to_string()))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .divide(Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .divide(Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(31)
                    .divide(Variant::VSingle(13.0))
                    .unwrap()
                {
                    Variant::VSingle(result) => assert_eq!(result, 2.384_615_4),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                assert_eq!(
                    Variant::VInteger(31)
                        .divide(Variant::VDouble(13.0))
                        .unwrap(),
                    Variant::VDouble(2.38461538461538)
                );
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .divide("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer_no_fraction() {
                match Variant::VInteger(42).divide(Variant::VInteger(2)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, 21),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_integer_fraction() {
                match Variant::VInteger(1).divide(Variant::VInteger(2)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 0.5),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VInteger(42).divide(Variant::VLong(2)).unwrap(),
                    Variant::VInteger(21)
                );
            }

            #[test]
            fn test_division_by_zero() {
                Variant::VInteger(1)
                    .divide(Variant::VSingle(0.0))
                    .expect_err("Division by zero");
                Variant::VInteger(1)
                    .divide(Variant::VDouble(0.0))
                    .expect_err("Division by zero");
                Variant::VInteger(1)
                    .divide(Variant::VInteger(0))
                    .expect_err("Division by zero");
                Variant::VInteger(1)
                    .divide(Variant::VLong(0))
                    .expect_err("Division by zero");
            }
        }

        mod long {
            use rusty_bit_vec::MAX_INTEGER;

            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(5).divide(Variant::VSingle(2.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 2.5),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(5).divide(Variant::VDouble(2.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 2.5),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .divide("hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                assert_eq!(
                    Variant::VLong(42).divide(Variant::VInteger(2)).unwrap(),
                    Variant::VInteger(21)
                );
            }

            #[test]
            fn test_long() {
                assert_eq!(
                    Variant::VLong(42).divide(Variant::VInteger(2)).unwrap(),
                    Variant::VInteger(21)
                );
            }

            #[test]
            fn test_long_exceeding_integer_range() {
                assert_eq!(
                    Variant::VLong(MAX_INTEGER as i64 * 4)
                        .divide(Variant::VInteger(2))
                        .unwrap(),
                    Variant::VLong(MAX_INTEGER as i64 * 2)
                );
            }

            #[test]
            fn test_division_by_zero() {
                Variant::VLong(1)
                    .divide(Variant::VSingle(0.0))
                    .expect_err("Division by zero");
                Variant::VLong(1)
                    .divide(Variant::VDouble(0.0))
                    .expect_err("Division by zero");
                Variant::VLong(1)
                    .divide(Variant::VInteger(0))
                    .expect_err("Division by zero");
                Variant::VLong(1)
                    .divide(Variant::VLong(0))
                    .expect_err("Division by zero");
            }
        }
    }

    mod compare {
        use super::*;

        fn assert_less(left: Variant, right: Variant) {
            assert_eq!(left.try_cmp(&right).unwrap(), Ordering::Less);
        }

        fn assert_equal(left: Variant, right: Variant) {
            assert_eq!(left.try_cmp(&right).unwrap(), Ordering::Equal);
        }

        fn assert_greater(left: Variant, right: Variant) {
            assert_eq!(left.try_cmp(&right).unwrap(), Ordering::Greater);
        }

        fn assert_err(left: Variant, right: Variant) {
            left.try_cmp(&right).expect_err("cannot compare");
        }

        #[test]
        fn test_single_to_single() {
            assert_less(Variant::from(1.0_f32), Variant::from(2.0_f32));
            assert_equal(Variant::from(3.0_f32), Variant::from(3.0_f32));
            assert_greater(Variant::from(5.0_f32), Variant::from(4.0_f32));
        }

        #[test]
        fn test_single_to_double() {
            assert_less(Variant::from(1.0_f32), Variant::from(2.0));
            assert_equal(Variant::from(3.0_f32), Variant::from(3.0));
            assert_greater(Variant::from(5.0_f32), Variant::from(4.0));
        }

        #[test]
        fn test_single_to_integer() {
            assert_less(Variant::from(1.0_f32), Variant::from(2));
            assert_less(Variant::from(1.9_f32), Variant::from(2));
            assert_equal(Variant::from(3.0_f32), Variant::from(3));
            assert_greater(Variant::from(5.0_f32), Variant::from(4));
            assert_greater(Variant::from(5.1_f32), Variant::from(5));
        }

        #[test]
        fn test_single_to_long() {
            assert_less(Variant::from(1.0_f32), Variant::from(2_i64));
            assert_equal(Variant::from(3.0_f32), Variant::from(3_i64));
            assert_greater(Variant::from(5.0_f32), Variant::from(4_i64));
        }

        #[test]
        fn test_numbers_to_string_both_ways() {
            assert_err(Variant::from(1.0_f32), Variant::from("hi"));
            assert_err(Variant::from(1.0), Variant::from("hi"));
            assert_err(Variant::from(1), Variant::from("hi"));
            assert_err(Variant::from(1_i64), Variant::from("hi"));
            assert_err(Variant::from("hi"), Variant::from(1.0_f32));
            assert_err(Variant::from("hi"), Variant::from(1.0));
            assert_err(Variant::from("hi"), Variant::from(1));
            assert_err(Variant::from("hi"), Variant::from(1_i64));
        }

        #[test]
        fn test_double_to_single() {
            assert_less(Variant::from(1.0), Variant::from(2.0_f32));
            assert_equal(Variant::from(3.0), Variant::from(3.0_f32));
            assert_greater(Variant::from(5.0), Variant::from(4.0_f32));
        }

        #[test]
        fn test_double_to_double() {
            assert_less(Variant::from(1.0), Variant::from(2.0));
            assert_equal(Variant::from(3.0), Variant::from(3.0));
            assert_greater(Variant::from(5.0), Variant::from(4.0));
        }

        #[test]
        fn test_double_to_integer() {
            assert_less(Variant::from(1.0), Variant::from(2));
            assert_equal(Variant::from(3.0), Variant::from(3));
            assert_greater(Variant::from(5.0), Variant::from(4));
        }

        #[test]
        fn test_double_to_long() {
            assert_less(Variant::from(1.0), Variant::from(2_i64));
            assert_equal(Variant::from(3.0), Variant::from(3_i64));
            assert_greater(Variant::from(5.0), Variant::from(4_i64));
        }

        #[test]
        fn test_integer_to_single() {
            assert_less(Variant::from(1), Variant::from(1.1_f32));
            assert_less(Variant::from(1), Variant::from(2.0_f32));
            assert_equal(Variant::from(3), Variant::from(3.0_f32));
            assert_greater(Variant::from(5), Variant::from(4.9_f32));
            assert_greater(Variant::from(5), Variant::from(4.0_f32));
        }

        #[test]
        fn test_integer_to_double() {
            assert_less(Variant::from(1), Variant::from(2.0));
            assert_equal(Variant::from(3), Variant::from(3.0));
            assert_greater(Variant::from(5), Variant::from(4.0));
        }

        #[test]
        fn test_integer_to_integer() {
            assert_less(Variant::from(1), Variant::from(2));
            assert_equal(Variant::from(3), Variant::from(3));
            assert_greater(Variant::from(5), Variant::from(4));
        }

        #[test]
        fn test_integer_to_long() {
            assert_less(Variant::from(1), Variant::from(2_i64));
            assert_equal(Variant::from(3), Variant::from(3_i64));
            assert_greater(Variant::from(5), Variant::from(4_i64));
        }

        #[test]
        fn test_long_to_single() {
            assert_less(Variant::from(1_i64), Variant::from(2.0_f32));
            assert_equal(Variant::from(3_i64), Variant::from(3.0_f32));
            assert_greater(Variant::from(5_i64), Variant::from(4.0_f32));
        }

        #[test]
        fn test_long_to_double() {
            assert_less(Variant::from(1_i64), Variant::from(2.0));
            assert_equal(Variant::from(3_i64), Variant::from(3.0));
            assert_greater(Variant::from(5_i64), Variant::from(4.0));
        }

        #[test]
        fn test_long_to_integer() {
            assert_less(Variant::from(1_i64), Variant::from(2));
            assert_equal(Variant::from(3_i64), Variant::from(3));
            assert_greater(Variant::from(5_i64), Variant::from(4));
        }

        #[test]
        fn test_long_to_long() {
            assert_less(Variant::from(1_i64), Variant::from(2_i64));
            assert_equal(Variant::from(3_i64), Variant::from(3_i64));
            assert_greater(Variant::from(5_i64), Variant::from(4_i64));
        }
    }
}
