use super::fit::FitToType;
use crate::common::FileHandle;
use crate::parser::{HasQualifier, TypeQualifier};
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
    VFileHandle(FileHandle),
}

pub const V_TRUE: Variant = Variant::VInteger(-1);
pub const V_FALSE: Variant = Variant::VInteger(0);

pub const MIN_INTEGER: i32 = -32768;
pub const MAX_INTEGER: i32 = 32767;
pub const MIN_LONG: i64 = -2147483648;
pub const MAX_LONG: i64 = 2147483647;

const INT_BITS: usize = 16;

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
            Err("Division by zero".to_string())
        } else {
            Ok(($nom / $div).fit_to_type())
        }
    };

    ($nom:expr, $div:expr, $cast:tt) => {
        if $div.approximate_eq(0) {
            Err("Division by zero".to_string())
        } else {
            Ok(($nom as $cast / $div as $cast).fit_to_type())
        }
    };
}

impl Variant {
    pub fn cmp(&self, other: &Self) -> Result<Ordering, String> {
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
                _ => Err("Type mismatch".to_string()),
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
            Variant::VFileHandle(s_left) => match other {
                Variant::VFileHandle(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err("Type mismatch".to_string()),
            },
        }
    }

    fn cmp_same_type_only(&self, other: &Self) -> Result<Ordering, String> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(ApproximateCmp::cmp(f_left, f_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(ApproximateCmp::cmp(d_left, d_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VFileHandle(s_left) => match other {
                Variant::VFileHandle(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err("Type mismatch".to_string()),
            },
        }
    }

    pub fn negate(&self) -> Result<Self, String> {
        match self {
            Variant::VSingle(n) => Ok(Variant::VSingle(-n)),
            Variant::VDouble(n) => Ok(Variant::VDouble(-n)),
            Variant::VInteger(n) => {
                if *n <= MIN_INTEGER {
                    // prevent converting -32768 to 32768
                    Err("Overflow".to_string())
                } else {
                    Ok(Variant::VInteger(-n))
                }
            }
            Variant::VLong(n) => {
                if *n <= MIN_LONG {
                    Err("Overflow".to_string())
                } else {
                    Ok(Variant::VLong(-n))
                }
            }
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn unary_not(&self) -> Result<Self, String> {
        match self {
            Variant::VSingle(f) => Ok(Variant::VSingle(-f.round() - 1.0)),
            Variant::VDouble(d) => Ok(Variant::VDouble(-d.round() - 1.0)),
            Variant::VInteger(n) => Ok(Variant::VInteger(-n - 1)),
            Variant::VLong(n) => Ok(Variant::VLong(-n - 1)),
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn plus(self, other: Self) -> Result<Self, String> {
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
                _ => Err("Type mismatch".to_string()),
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
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn minus(self, other: Self) -> Result<Self, String> {
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
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn multiply(self, other: Self) -> Result<Self, String> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(f_left * f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(f_left as f64 * d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(f_left * i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(f_left * l_right as f32)),
                _ => Err("Type mismatch".to_string()),
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
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn divide(self, other: Self) -> Result<Self, String> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => div!(f_left, f_right),
                Variant::VDouble(d_right) => div!(f_left, d_right, f64),
                Variant::VInteger(i_right) => div!(f_left, i_right, f32),
                Variant::VLong(l_right) => div!(f_left, l_right, f32),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VSingle(f_right) => div!(d_left, f_right, f64),
                Variant::VDouble(d_right) => div!(d_left, d_right),
                Variant::VInteger(i_right) => div!(d_left, i_right, f64),
                Variant::VLong(l_right) => div!(d_left, l_right, f64),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VSingle(f_right) => div!(i_left, f_right, f32),
                Variant::VDouble(d_right) => div!(i_left, d_right, f64),
                Variant::VInteger(i_right) => div!(i_left, i_right, f32),
                Variant::VLong(l_right) => div!(i_left, l_right, f32),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VLong(l_left) => match other {
                Variant::VSingle(f_right) => div!(l_left, f_right, f32),
                Variant::VDouble(d_right) => div!(l_left, d_right, f64),
                Variant::VInteger(i_right) => div!(l_left, i_right, f32),
                Variant::VLong(l_right) => div!(l_left, l_right, f32),
                _ => Err("Type mismatch".to_string()),
            },
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn and(self, other: Self) -> Result<Self, String> {
        match self {
            Variant::VInteger(a) => match other {
                Variant::VInteger(b) => Ok(Variant::VInteger(from_bits(and_bits(
                    to_bits(a),
                    to_bits(b),
                )))),
                _ => Err("Type mismatch".to_string()),
            },
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn or(self, other: Self) -> Result<Self, String> {
        match self {
            Variant::VInteger(a) => match other {
                Variant::VInteger(b) => Ok(Variant::VInteger(from_bits(or_bits(
                    to_bits(a),
                    to_bits(b),
                )))),
                _ => Err("Type mismatch".to_string()),
            },
            _ => Err("Type mismatch".to_string()),
        }
    }

    pub fn default_variant(type_qualifier: TypeQualifier) -> Variant {
        match type_qualifier {
            TypeQualifier::BangSingle => Variant::VSingle(0.0),
            TypeQualifier::HashDouble => Variant::VDouble(0.0),
            TypeQualifier::DollarString => Variant::VString(String::new()),
            TypeQualifier::PercentInteger => Variant::VInteger(0),
            TypeQualifier::AmpersandLong => Variant::VLong(0),
            TypeQualifier::FileHandle => Variant::VFileHandle(FileHandle::default()),
        }
    }

    /// Demands that the variant holds an integer and returns the integer value.
    /// Panics if the variant is not an integer.
    pub fn demand_integer(self) -> i32 {
        match self {
            Variant::VInteger(v) => v,
            _ => panic!("not an integer variant"),
        }
    }

    pub fn demand_file_handle(self) -> FileHandle {
        match self {
            Variant::VFileHandle(v) => v,
            _ => panic!("not a file handle variant"),
        }
    }

    pub fn demand_string(self) -> String {
        match self {
            Variant::VString(s) => s,
            _ => panic!("not a string variant"),
        }
    }
}

impl HasQualifier for Variant {
    fn qualifier(&self) -> TypeQualifier {
        match self {
            Variant::VSingle(_) => TypeQualifier::BangSingle,
            Variant::VDouble(_) => TypeQualifier::HashDouble,
            Variant::VString(_) => TypeQualifier::DollarString,
            Variant::VInteger(_) => TypeQualifier::PercentInteger,
            Variant::VLong(_) => TypeQualifier::AmpersandLong,
            Variant::VFileHandle(_) => TypeQualifier::FileHandle,
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

impl From<&String> for Variant {
    fn from(s: &String) -> Self {
        Variant::VString(s.to_owned())
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

impl TryFrom<&Variant> for bool {
    type Error = String;

    fn try_from(value: &Variant) -> Result<bool, String> {
        match value {
            Variant::VSingle(n) => Ok(*n != 0.0),
            Variant::VDouble(n) => Ok(*n != 0.0),
            Variant::VInteger(n) => Ok(*n != 0),
            Variant::VLong(n) => Ok(*n != 0),
            _ => Err("Type mismatch".to_string()),
        }
    }
}

impl TryFrom<Variant> for bool {
    type Error = String;

    fn try_from(value: Variant) -> Result<bool, String> {
        bool::try_from(&value)
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

fn to_bits(a: i32) -> [bool; INT_BITS] {
    let mut result: [bool; INT_BITS] = [false; INT_BITS];
    let mut x: i32 = a;
    if x > 0 {
        let mut idx = 0;
        while x > 0 && idx < INT_BITS {
            result[idx] = (x & 1) == 1;
            x = x >> 1;
            idx += 1;
        }
    } else if x < 0 {
        x = -x - 1;
        result = [true; INT_BITS];
        let mut idx = 0;
        while x > 0 && idx < INT_BITS {
            result[idx] = (x & 1) == 0;
            x = x >> 1;
            idx += 1;
        }
    }
    result
}

fn from_bits(bits: [bool; INT_BITS]) -> i32 {
    let mut x: i32 = 0;
    let sign = bits[INT_BITS - 1];
    let mut idx = INT_BITS - 1;
    while idx > 0 {
        x = x << 1;
        idx -= 1;
        if bits[idx] != sign {
            x = x | 1;
        }
    }
    if sign {
        -x - 1
    } else {
        x
    }
}

fn and_bits(a: [bool; INT_BITS], b: [bool; INT_BITS]) -> [bool; INT_BITS] {
    let mut c: [bool; INT_BITS] = [false; INT_BITS];
    for i in 0..INT_BITS {
        c[i] = a[i] && b[i];
    }
    c
}

fn or_bits(a: [bool; INT_BITS], b: [bool; INT_BITS]) -> [bool; INT_BITS] {
    let mut c: [bool; INT_BITS] = [false; INT_BITS];
    for i in 0..INT_BITS {
        c[i] = a[i] || b[i];
    }
    c
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
            assert_eq!(true, bool::try_from(Variant::from(1.0_f32)).unwrap());
            assert_eq!(false, bool::try_from(Variant::from(0.0_f32)).unwrap());
            assert_eq!(true, bool::try_from(Variant::from(1.0)).unwrap());
            assert_eq!(false, bool::try_from(Variant::from(0.0)).unwrap());
            bool::try_from(Variant::from("hi")).expect_err("should not convert from string");
            bool::try_from(Variant::from("")).expect_err("should not convert from string");
            assert_eq!(true, bool::try_from(Variant::from(42)).unwrap());
            assert_eq!(false, bool::try_from(Variant::from(0)).unwrap());
            assert_eq!(true, bool::try_from(Variant::from(42_i64)).unwrap());
            assert_eq!(false, bool::try_from(Variant::from(0_i64)).unwrap());
            assert_eq!(true, bool::try_from(V_TRUE).unwrap());
            assert_eq!(false, bool::try_from(V_FALSE).unwrap());
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

    mod bits {
        use super::*;

        #[test]
        fn test_positive_bits() {
            let mut expected_bits: [bool; INT_BITS] = [false; INT_BITS];

            // 0 | 0 0 0
            assert_eq!(to_bits(0), expected_bits);

            // 0 | 0 0 1
            expected_bits[0] = true;
            assert_eq!(to_bits(1), expected_bits);

            // 0 | 0 1 0
            expected_bits[0] = false;
            expected_bits[1] = true;
            assert_eq!(to_bits(2), expected_bits);

            // 0 | 0 1 1
            expected_bits[0] = true;
            assert_eq!(to_bits(3), expected_bits);

            // 0 | 1 0 0
            expected_bits[0] = false;
            expected_bits[1] = false;
            expected_bits[2] = true;
            assert_eq!(to_bits(4), expected_bits);

            // 0 | 1 0 1
            expected_bits[0] = true;
            assert_eq!(to_bits(5), expected_bits);
        }

        #[test]
        fn test_negative_bits() {
            let mut expected_bits: [bool; INT_BITS] = [true; INT_BITS];

            // 1 | 1 1 1
            assert_eq!(to_bits(-1), expected_bits);

            // 1 | 1 1 0
            expected_bits[0] = false;
            assert_eq!(to_bits(-2), expected_bits);

            // 1 | 1 0 1
            expected_bits[0] = true;
            expected_bits[1] = false;
            assert_eq!(to_bits(-3), expected_bits);

            // 1 | 1 0 0
            expected_bits[0] = false;
            assert_eq!(to_bits(-4), expected_bits);

            // 1 | 0 1 1
            expected_bits[0] = true;
            expected_bits[1] = true;
            expected_bits[2] = false;
            assert_eq!(to_bits(-5), expected_bits);
        }

        #[test]
        fn test_from_to_bits() {
            for i in -5..6 {
                assert_eq!(i, from_bits(to_bits(i)));
            }
        }

        #[test]
        fn test_and_bits() {
            assert_eq!(4, from_bits(and_bits(to_bits(5), to_bits(-2))));
            assert_eq!(2, from_bits(and_bits(to_bits(-5), to_bits(2))));
        }
    }
}
