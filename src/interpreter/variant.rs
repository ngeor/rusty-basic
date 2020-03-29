use crate::common::Result;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    VSingle(f32),
    VDouble(f64),
    VString(String),
    VInteger(i32),
    VLong(i64),
}

pub const V_TRUE: Variant = Variant::VInteger(-1);
pub const V_FALSE: Variant = Variant::VInteger(0);

fn partial_cmp_to_result<T>(left: &T, right: &T) -> Result<Ordering>
where
    T: PartialOrd + Display + Copy,
{
    match left.partial_cmp(right) {
        Some(o) => Ok(o),
        _ => Err(format!("Could not compare {} with {}", left, right)),
    }
}

impl Variant {
    pub fn cmp(&self, other: &Self) -> Result<Ordering> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => partial_cmp_to_result(f_left, f_right),
                Variant::VDouble(d_right) => partial_cmp_to_result(&(*f_left as f64), d_right),
                Variant::VInteger(i_right) => partial_cmp_to_result(f_left, &(*i_right as f32)),
                Variant::VLong(l_right) => partial_cmp_to_result(f_left, &(*l_right as f32)),
                _ => other.cmp(self).map(|x| x.reverse())
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => partial_cmp_to_result(d_left, d_right),
                Variant::VInteger(i_right) => partial_cmp_to_result(d_left, &(*i_right as f64)),
                Variant::VLong(l_right) => partial_cmp_to_result(d_left, &(*l_right as f64)),
                _ => other.cmp(self).map(|x| x.reverse())
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                Variant::VLong(l_right) => partial_cmp_to_result(&(*i_left as i64), l_right),
                _ => other.cmp(self).map(|x| x.reverse())
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(l_left.cmp(l_right)),
                _ => other.cmp(self).map(|x| x.reverse())
            },
        }
    }

    pub fn negate(&self) -> Self {
        match self {
            Variant::VSingle(n) => Variant::VSingle(-n),
            Variant::VDouble(n) => Variant::VDouble(-n),
            Variant::VInteger(n) => Variant::VInteger(-n),
            Variant::VLong(n) => Variant::VLong(-n),
            _ => unimplemented!()
        }
    }

    pub fn plus(&self, other: &Self) -> Result<Self> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(*f_left + *f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*f_left as f64 + *d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(*f_left + *i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(*f_left + *l_right as f32)),
                _ => other.plus(self),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*d_left + *d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(*d_left + *i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(*d_left + *l_right as f64)),
                _ => other.plus(self),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(Variant::VString(format!("{}{}", s_left, s_right))),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(*i_left + *i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*i_left as i64 + *l_right)),
                _ => other.plus(self),
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(Variant::VLong(*l_left + *l_right)),
                _ => other.plus(self)
            },
        }
    }

    pub fn minus(&self, other: &Self) -> Result<Self> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(*f_left - *f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*f_left as f64 - *d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(*f_left - *i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(*f_left - *l_right as f32)),
                _ => other.minus(self).map(|x| x.negate()),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*d_left - *d_right)),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(*d_left - *i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(*d_left - *l_right as f64)),
                _ => other.minus(self).map(|x| x.negate()),
            },
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(*i_left - *i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*i_left as i64 - *l_right)),
                _ => other.minus(self).map(|x| x.negate())
            },
            Variant::VLong(l_left) => match other {
                Variant::VLong(l_right) => Ok(Variant::VLong(*l_left - *l_right)),
                _ => other.minus(self).map(|x| x.negate())
            },
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

    fn try_from(value: &Variant) -> Result<bool> {
        match value {
            Variant::VSingle(n) => Ok(*n != 0.0),
            Variant::VDouble(n) => Ok(*n != 0.0),
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(n) => Ok(*n != 0),
            Variant::VLong(n) => Ok(*n != 0),
        }
    }
}

impl TryFrom<Variant> for bool {
    type Error = String;

    fn try_from(value: Variant) -> Result<bool> {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON_SINGLE: f32 = 0.00001;
    const EPSILON_DOUBLE: f64 = 0.00001;

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
                match Variant::VSingle(1.1).plus(&Variant::VSingle(2.4)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.5).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VSingle(1.1).plus(&Variant::VDouble(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .plus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VSingle(1.1).plus(&Variant::VInteger(2)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.1).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VSingle(1.1).plus(&Variant::VLong(2)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.1).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VDouble(1.1).plus(&Variant::VSingle(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VDouble(1.1).plus(&Variant::VDouble(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .plus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VDouble(1.1).plus(&Variant::VInteger(2)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.1).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VDouble(1.1).plus(&Variant::VLong(2)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.1).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .plus(&Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .plus(&Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                match Variant::VString("hello".to_string())
                    .plus(&Variant::VString(" world".to_string()))
                    .unwrap()
                {
                    Variant::VString(result) => assert_eq!(result, "hello world"),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .plus(&Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .plus(&Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(1)
                    .plus(&Variant::VSingle(0.5))
                    .unwrap()
                {
                    Variant::VSingle(result) => assert_eq!(result, 1.5),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VInteger(1)
                    .plus(&Variant::VDouble(0.6))
                    .unwrap()
                {
                    Variant::VDouble(result) => assert_eq!(result, 1.6),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .plus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VInteger(42).plus(&Variant::VInteger(2)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VInteger(42).plus(&Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod long {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(1).plus(&Variant::VSingle(2.0)).unwrap() {
                    Variant::VSingle(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(1).plus(&Variant::VDouble(2.0)).unwrap() {
                    Variant::VDouble(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .plus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VLong(42).plus(&Variant::VInteger(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 44),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VLong(42).plus(&Variant::VLong(2)).unwrap() {
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
                match Variant::VSingle(5.9).minus(&Variant::VSingle(2.4)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.5).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VSingle(5.9).minus(&Variant::VDouble(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VSingle(5.1)
                    .minus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VSingle(5.1).minus(&Variant::VInteger(2)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.1).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VSingle(5.1).minus(&Variant::VLong(2)).unwrap() {
                    Variant::VSingle(result) => assert!((result - 3.1).abs() <= EPSILON_SINGLE),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod double {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VDouble(5.9).minus(&Variant::VSingle(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VDouble(5.9).minus(&Variant::VDouble(2.4)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.5).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VDouble(5.1)
                    .minus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VDouble(5.1).minus(&Variant::VInteger(2)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.1).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VDouble(5.1).minus(&Variant::VLong(2)).unwrap() {
                    Variant::VDouble(result) => assert!((result - 3.1).abs() <= EPSILON_DOUBLE),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod string {
            use super::*;

            #[test]
            fn test_single() {
                Variant::VString("hello".to_string())
                    .minus(&Variant::VSingle(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_double() {
                Variant::VString("hello".to_string())
                    .minus(&Variant::VDouble(1.2))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_string() {
                Variant::VString("hello".to_string())
                    .minus(&Variant::VString("world".to_string()))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                Variant::VString("hello".to_string())
                    .minus(&Variant::VInteger(42))
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_long() {
                Variant::VString("hello".to_string())
                    .minus(&Variant::VLong(42))
                    .expect_err("Type mismatch");
            }
        }

        mod integer {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VInteger(31)
                    .minus(&Variant::VSingle(13.0))
                    .unwrap()
                {
                    Variant::VSingle(result) => assert_eq!(result, 18.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VInteger(31)
                    .minus(&Variant::VDouble(13.0))
                    .unwrap()
                {
                    Variant::VDouble(result) => assert_eq!(result, 18.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VInteger(42)
                    .minus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VInteger(42).minus(&Variant::VInteger(2)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VInteger(42).minus(&Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }
        }

        mod long {
            use super::*;

            #[test]
            fn test_single() {
                match Variant::VLong(5)
                    .minus(&Variant::VSingle(2.0))
                    .unwrap()
                {
                    Variant::VSingle(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                match Variant::VLong(5)
                    .minus(&Variant::VDouble(2.0))
                    .unwrap()
                {
                    Variant::VDouble(result) => assert_eq!(result, 3.0),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_string() {
                Variant::VLong(42)
                    .minus(&"hi".into())
                    .expect_err("Type mismatch");
            }

            #[test]
            fn test_integer() {
                match Variant::VLong(42).minus(&Variant::VInteger(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_long() {
                match Variant::VLong(42).minus(&Variant::VLong(2)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, 40),
                    _ => panic!("assertion failed"),
                }
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
