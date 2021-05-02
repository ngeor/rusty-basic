use super::fit::FitToType;
use super::UserDefinedTypeValue;
use crate::common::QError;
use crate::parser::TypeQualifier;
use crate::variant::bits::{bytes_to_i32, i32_to_bytes};
use crate::variant::{qb_and, qb_or, AsciiSize, QBNumberCast, VArray};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

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

pub const V_TRUE: Variant = Variant::VInteger(-1);
pub const V_FALSE: Variant = Variant::VInteger(0);

pub const MIN_INTEGER: i32 = -32768;
pub const MAX_INTEGER: i32 = 32767;
pub const MIN_LONG: i64 = -2147483648;
pub const MAX_LONG: i64 = 2147483647;

trait ApproximateCmp {
    fn cmp(left: &Self, right: &Self) -> Ordering;
}

impl ApproximateCmp for f32 {
    fn cmp(left: &f32, right: &f32) -> Ordering {
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
    fn cmp(left: &f64, right: &f64) -> Ordering {
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
        (self - right as f32).abs() < 0.00001
    }
}

impl ApproximateEqToInt for f64 {
    fn approximate_eq(self, right: i32) -> bool {
        (self - right as f64).abs() < 0.00001
    }
}

macro_rules! div {
    ($nom:expr, $div:expr) => {
        if $div.approximate_eq(0) {
            Err(QError::DivisionByZero)
        } else {
            Ok(($nom / $div).fit_to_type())
        }
    };

    ($nom:expr, $div:expr, $cast:tt) => {
        if $div.approximate_eq(0) {
            Err(QError::DivisionByZero)
        } else {
            Ok(($nom as $cast / $div as $cast).fit_to_type())
        }
    };
}

impl Variant {
    pub fn cmp(&self, other: &Self) -> Result<Ordering, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(ApproximateCmp::cmp(f_left, f_right)),
                Variant::VDouble(d_right) => Ok(ApproximateCmp::cmp(&(*f_left as f64), d_right)),
                Variant::VInteger(i_right) => Ok(ApproximateCmp::cmp(f_left, &(*i_right as f32))),
                Variant::VLong(l_right) => Ok(ApproximateCmp::cmp(f_left, &(*l_right as f32))),
                _ => other.cmp(self).map(|x| x.reverse()),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(ApproximateCmp::cmp(d_left, d_right)),
                Variant::VInteger(i_right) => Ok(ApproximateCmp::cmp(d_left, &(*i_right as f64))),
                Variant::VLong(l_right) => Ok(ApproximateCmp::cmp(d_left, &(*l_right as f64))),
                _ => other.cmp(self).map(|x| x.reverse()),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                Variant::VLong(l_right) => Ok((*i_left as i64).cmp(l_right)),
                _ => other.cmp(self).map(|x| x.reverse()),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => other.cmp(self).map(|x| x.reverse()),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    fn cmp_same_type_only(&self, other: &Self) -> Result<Ordering, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(ApproximateCmp::cmp(f_left, f_right)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(ApproximateCmp::cmp(d_left, d_right)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => Err(QError::TypeMismatch),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn negate(self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(n) => Ok(Variant::VSingle(-n)),
            Variant::VDouble(n) => Ok(Variant::VDouble(-n)),
            Variant::VInteger(n) => {
                if n <= MIN_INTEGER {
                    // prevent converting -32768 to 32768
                    Err(QError::Overflow)
                } else {
                    Ok(Variant::VInteger(-n))
                }
            }
            Variant::VLong(n) => {
                if n <= MIN_LONG {
                    Err(QError::Overflow)
                } else {
                    Ok(Variant::VLong(-n))
                }
            }
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn unary_not(self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f) => Ok(Variant::VSingle(-f.round() - 1.0)),
            Variant::VDouble(d) => Ok(Variant::VDouble(-d.round() - 1.0)),
            Variant::VInteger(n) => Ok(Variant::VInteger(-n - 1)),
            Variant::VLong(n) => Ok(Variant::VLong(-n - 1)),
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn plus(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(f_left + f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(f_left as f64 + d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(f_left + i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(f_left + l_right as f32)),
                _ => other.plus(self),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(Variant::VDouble(d_left + d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(d_left + i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(d_left + l_right as f64)),
                _ => other.plus(self),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(Variant::VString(format!("{}{}", s_left, s_right))),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(i_left + i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(i_left as i64 + l_right)),
                _ => other.plus(self),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(Variant::VLong(l_left + l_right)),
                _ => other.plus(self),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn minus(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(f_left - f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(f_left as f64 - d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(f_left - i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(f_left - l_right as f32)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(Variant::VDouble(d_left - d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(d_left - i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(d_left - l_right as f64)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(i_left - i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(i_left as i64 - l_right)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(Variant::VLong(l_left - l_right)),
                _ => other.minus(self).and_then(|x| x.negate()),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn multiply(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(f_left * f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(f_left as f64 * d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(f_left * i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(f_left * l_right as f32)),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(Variant::VDouble(d_left * d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(d_left * i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(d_left * l_right as f64)),
                _ => other.multiply(self),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(i_left * i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(i_left as i64 * l_right)),
                _ => other.multiply(self),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(Variant::VLong(l_left * l_right)),
                _ => other.multiply(self),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn divide(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => div!(f_left, f_right),
                Variant::VDouble(d_right) => div!(f_left, d_right, f64),
                Variant::VInteger(i_right) => div!(f_left, i_right, f32),
                Variant::VLong(l_right) => div!(f_left, l_right, f32),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VSingle(f_right) => div!(d_left, f_right, f64),
                Variant::VDouble(d_right) => div!(d_left, d_right),
                Variant::VInteger(i_right) => div!(d_left, i_right, f64),
                Variant::VLong(l_right) => div!(d_left, l_right, f64),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VSingle(f_right) => div!(i_left, f_right, f32),
                Variant::VDouble(d_right) => div!(i_left, d_right, f64),
                Variant::VInteger(i_right) => div!(i_left, i_right, f32),
                Variant::VLong(l_right) => div!(i_left, l_right, f32),
                _ => Err(QError::TypeMismatch),
            },
            Variant::VLong(l_left) => match other {
                Variant::VSingle(f_right) => div!(l_left, f_right, f32),
                Variant::VDouble(d_right) => div!(l_left, d_right, f64),
                Variant::VInteger(i_right) => div!(l_left, i_right, f32),
                Variant::VLong(l_right) => div!(l_left, l_right, f32),
                _ => Err(QError::TypeMismatch),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn modulo(self, other: Self) -> Result<Self, QError> {
        let round_left = self.round()?;
        let round_right = other.round()?;
        if round_right.is_approximately_zero()? {
            Err(QError::DivisionByZero)
        } else {
            match round_left {
                Variant::VInteger(i_left) => match round_right {
                    Variant::VInteger(i_right) => Ok(Variant::VInteger(i_left % i_right)),
                    Variant::VLong(_) => Err(QError::Overflow),
                    _ => Err(QError::TypeMismatch),
                },
                Variant::VLong(_) => Err(QError::Overflow),
                _ => Err(QError::TypeMismatch),
            }
        }
    }

    fn round(self) -> Result<Self, QError> {
        match self {
            Variant::VSingle(f) => Ok(f.round().fit_to_type()),
            Variant::VDouble(d) => Ok(d.round().fit_to_type()),
            Variant::VInteger(_) | Variant::VLong(_) => Ok(self),
            _ => Err(QError::TypeMismatch),
        }
    }

    fn is_approximately_zero(&self) -> Result<bool, QError> {
        match self {
            Variant::VSingle(f) => Ok((*f).approximate_eq(0)),
            Variant::VDouble(d) => Ok((*d).approximate_eq(0)),
            Variant::VInteger(i) => Ok(*i == 0),
            Variant::VLong(l) => Ok(*l == 0),
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn and(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VInteger(a) => match other {
                Variant::VInteger(b) => Ok(Variant::VInteger(qb_and(a, b))),
                _ => Err(QError::TypeMismatch),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn or(self, other: Self) -> Result<Self, QError> {
        match self {
            Variant::VInteger(a) => match other {
                Variant::VInteger(b) => Ok(Variant::VInteger(qb_or(a, b))),
                _ => Err(QError::TypeMismatch),
            },
            _ => Err(QError::TypeMismatch),
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Self::VArray(_) => true,
            _ => false,
        }
    }

    pub fn peek_non_array(&self, address: usize) -> Result<u8, QError> {
        match self {
            Self::VInteger(i) => {
                let bytes = i32_to_bytes(*i);
                Ok(bytes[address])
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn poke_non_array(&mut self, address: usize, byte_value: u8) -> Result<(), QError> {
        match self {
            Self::VInteger(i) => {
                let mut bytes = i32_to_bytes(*i);
                bytes[address] = byte_value;
                *i = bytes_to_i32(bytes);
                Ok(())
            }
            _ => {
                todo!()
            }
        }
    }
}

impl AsciiSize for Variant {
    fn ascii_size(&self) -> usize {
        match self {
            Self::VInteger(_) => 2,
            Self::VLong(_) | Self::VSingle(_) => 4,
            Self::VDouble(_) => 8,
            Self::VString(s) => s.chars().count(),
            Self::VArray(v_array) => v_array.ascii_size(),
            Self::VUserDefined(user_defined_type_value) => user_defined_type_value.ascii_size(),
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
            Variant::VSingle(n) => write!(f, "{}", n),
            Variant::VDouble(n) => write!(f, "{}", n),
            Variant::VString(s) => write!(f, "{}", s),
            Variant::VInteger(n) => write!(f, "{}", n),
            Variant::VLong(n) => write!(f, "{}", n),
            _ => Err(std::fmt::Error),
        }
    }
}

// ========================================================
// Creating the default variant
// ========================================================

impl From<TypeQualifier> for Variant {
    fn from(type_qualifier: TypeQualifier) -> Self {
        match type_qualifier {
            TypeQualifier::BangSingle => Self::VSingle(0.0),
            TypeQualifier::HashDouble => Self::VDouble(0.0),
            TypeQualifier::DollarString => Self::VString(String::new()),
            TypeQualifier::PercentInteger => Self::VInteger(0),
            TypeQualifier::AmpersandLong => Self::VLong(0),
        }
    }
}

// ========================================================
// Try to get a type qualifier
// ========================================================

impl TryFrom<&Variant> for TypeQualifier {
    type Error = QError;

    fn try_from(value: &Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::VSingle(_) => Ok(TypeQualifier::BangSingle),
            Variant::VDouble(_) => Ok(TypeQualifier::HashDouble),
            Variant::VString(_) => Ok(TypeQualifier::DollarString),
            Variant::VInteger(_) => Ok(TypeQualifier::PercentInteger),
            Variant::VLong(_) => Ok(TypeQualifier::AmpersandLong),
            _ => Err(QError::TypeMismatch),
        }
    }
}

// ========================================================
// Convert from standard types to Variant
// ========================================================

impl From<f32> for Variant {
    fn from(f: f32) -> Self {
        Variant::VSingle(f)
    }
}

impl From<f64> for Variant {
    fn from(f: f64) -> Self {
        Variant::VDouble(f)
    }
}

impl From<String> for Variant {
    fn from(s: String) -> Self {
        Variant::VString(s)
    }
}

impl From<&str> for Variant {
    fn from(s: &str) -> Self {
        Variant::VString(s.to_string())
    }
}

impl From<i32> for Variant {
    fn from(i: i32) -> Self {
        Variant::VInteger(i)
    }
}

impl From<i64> for Variant {
    fn from(i: i64) -> Self {
        Variant::VLong(i)
    }
}

impl From<bool> for Variant {
    fn from(b: bool) -> Self {
        if b {
            V_TRUE
        } else {
            V_FALSE
        }
    }
}

// ========================================================
// Convert from Variant to standard types
// ========================================================

impl Variant {
    /// Gets a `str` reference from this Variant.
    ///
    /// Panics if the variant is not of string type.
    ///
    /// Use it only at runtime if the linter has guaranteed the type.
    pub fn to_str_unchecked(&self) -> &str {
        match self {
            Variant::VString(s) => s,
            _ => panic!("Variant was not a string {:?}", self),
        }
    }
}

impl QBNumberCast<bool> for Variant {
    fn try_cast(&self) -> Result<bool, QError> {
        match self {
            Variant::VSingle(n) => Ok(*n != 0.0),
            Variant::VDouble(n) => Ok(*n != 0.0),
            Variant::VInteger(n) => Ok(*n != 0),
            Variant::VLong(n) => Ok(*n != 0),
            _ => Err(QError::TypeMismatch),
        }
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

    mod try_from {
        use super::*;

        #[test]
        fn test_bool_try_from() {
            assert_eq!(true, bool_try_from(Variant::from(1.0_f32)).unwrap());
            assert_eq!(false, bool_try_from(Variant::from(0.0_f32)).unwrap());
            assert_eq!(true, bool_try_from(Variant::from(1.0)).unwrap());
            assert_eq!(false, bool_try_from(Variant::from(0.0)).unwrap());
            bool_try_from(Variant::from("hi")).expect_err("should not convert from string");
            bool_try_from(Variant::from("")).expect_err("should not convert from string");
            assert_eq!(true, bool_try_from(Variant::from(42)).unwrap());
            assert_eq!(false, bool_try_from(Variant::from(0)).unwrap());
            assert_eq!(true, bool_try_from(Variant::from(42_i64)).unwrap());
            assert_eq!(false, bool_try_from(Variant::from(0_i64)).unwrap());
            assert_eq!(true, bool_try_from(V_TRUE).unwrap());
            assert_eq!(false, bool_try_from(V_FALSE).unwrap());
        }

        fn bool_try_from(v: Variant) -> Result<bool, QError> {
            v.try_cast()
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
                    Variant::VSingle(result) => assert_eq!(result, 2.38461538461538),
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
            assert_eq!(left.cmp(&right).unwrap(), Ordering::Less);
        }

        fn assert_equal(left: Variant, right: Variant) {
            assert_eq!(left.cmp(&right).unwrap(), Ordering::Equal);
        }

        fn assert_greater(left: Variant, right: Variant) {
            assert_eq!(left.cmp(&right).unwrap(), Ordering::Greater);
        }

        fn assert_err(left: Variant, right: Variant) {
            left.cmp(&right).expect_err("cannot compare");
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
