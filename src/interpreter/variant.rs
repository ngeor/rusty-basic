use crate::common::Result;

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

impl Variant {
    pub fn is_true(&self) -> Result<bool> {
        match self {
            Variant::VSingle(n) => Ok(*n != 0.0),
            Variant::VDouble(n) => Ok(*n != 0.0),
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(n) => Ok(*n != 0),
            Variant::VLong(n) => Ok(*n != 0),
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Variant::VSingle(n) => format!("{}", n),
            Variant::VDouble(n) => format!("{}", n),
            Variant::VString(s) => s.clone(),
            Variant::VInteger(n) => format!("{}", n),
            Variant::VLong(n) => format!("{}", n),
        }
    }

    pub fn to_int(&self) -> Result<i32> {
        match self {
            Variant::VSingle(f) => Ok(f.round() as i32),
            Variant::VString(s) => s
                .parse::<i32>()
                .map_err(|e| format!("Could not convert {} to a number: {}", s, e)),
            Variant::VInteger(i) => Ok(*i),
            _ => unimplemented!(),
        }
    }

    pub fn compare_to(&self, other: &Variant) -> Result<std::cmp::Ordering> {
        match self {
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                _ => unimplemented!(),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(i_left.cmp(i_right)),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn plus(&self, other: &Variant) -> Result<Variant> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(*f_left + *f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*f_left as f64 + *d_right)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(*f_left + *i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(*f_left + *l_right as f32)),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VDouble(*d_left + *f_right as f64)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*d_left + *d_right)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(*d_left + *i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(*d_left + *l_right as f64)),
            },
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(Variant::VString(format!("{}{}", s_left, s_right))),
                _ => Err("Type mismatch".to_string()),
            },
            Variant::VInteger(i_left) => match other {
                Variant::VSingle(f_right) => {
                    Ok(Variant::VInteger(*i_left + f_right.round() as i32))
                }
                Variant::VDouble(d_right) => {
                    Ok(Variant::VInteger(*i_left + d_right.round() as i32))
                }
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VInteger(*i_left + *i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*i_left as i64 + *l_right)),
            },
            Variant::VLong(l_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VLong(*l_left + f_right.round() as i64)),
                Variant::VDouble(d_right) => Ok(Variant::VLong(*l_left + d_right.round() as i64)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VLong(*l_left + *i_right as i64)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*l_left + *l_right)),
            },
        }
    }

    pub fn minus(&self, other: &Variant) -> Result<Variant> {
        match self {
            Variant::VSingle(f_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VSingle(*f_left - *f_right)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*f_left as f64 - *d_right)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VSingle(*f_left - *i_right as f32)),
                Variant::VLong(l_right) => Ok(Variant::VSingle(*f_left - *l_right as f32)),
            },
            Variant::VDouble(d_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VDouble(*d_left - *f_right as f64)),
                Variant::VDouble(d_right) => Ok(Variant::VDouble(*d_left - *d_right)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VDouble(*d_left - *i_right as f64)),
                Variant::VLong(l_right) => Ok(Variant::VDouble(*d_left - *l_right as f64)),
            },
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(i_left) => match other {
                Variant::VSingle(f_right) => {
                    Ok(Variant::VInteger(*i_left - f_right.round() as i32))
                }
                Variant::VDouble(d_right) => {
                    Ok(Variant::VInteger(*i_left - d_right.round() as i32))
                }
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VInteger(*i_left - *i_right)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*i_left as i64 - *l_right)),
            },
            Variant::VLong(l_left) => match other {
                Variant::VSingle(f_right) => Ok(Variant::VLong(*l_left - f_right.round() as i64)),
                Variant::VDouble(d_right) => Ok(Variant::VLong(*l_left - d_right.round() as i64)),
                Variant::VString(_) => Err("Type mismatch".to_string()),
                Variant::VInteger(i_right) => Ok(Variant::VLong(*l_left - *i_right as i64)),
                Variant::VLong(l_right) => Ok(Variant::VLong(*l_left - *l_right)),
            },
        }
    }

    pub fn try_cast_to_single(&self) -> Result<Variant> {
        match *self {
            Variant::VSingle(f) => Ok(Variant::VSingle(f)),
            Variant::VDouble(d) => Ok(Variant::VSingle(d as f32)),
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(i) => Ok(Variant::VSingle(i as f32)),
            Variant::VLong(l) => Ok(Variant::VSingle(l as f32)),
        }
    }

    pub fn try_cast_to_double(&self) -> Result<Variant> {
        match self {
            Variant::VSingle(f) => Ok(Variant::VDouble(*f as f64)),
            Variant::VDouble(d) => Ok(Variant::VDouble(*d as f64)),
            Variant::VString(_) => Err("Type mismatch".to_string()),
            Variant::VInteger(i) => Ok(Variant::VDouble(*i as f64)),
            Variant::VLong(l) => Ok(Variant::VDouble(*l as f64)),
        }
    }
}

impl From<f32> for Variant {
    fn from(f: f32) -> Variant {
        Variant::VSingle(f)
    }
}

impl From<f64> for Variant {
    fn from(f: f64) -> Variant {
        Variant::VDouble(f)
    }
}

impl From<String> for Variant {
    fn from(s: String) -> Variant {
        Variant::VString(s)
    }
}

impl From<&String> for Variant {
    fn from(s: &String) -> Variant {
        Variant::VString(s.to_owned())
    }
}

impl From<&str> for Variant {
    fn from(s: &str) -> Variant {
        Variant::VString(s.to_string())
    }
}

impl From<i32> for Variant {
    fn from(i: i32) -> Variant {
        Variant::VInteger(i)
    }
}

impl From<i64> for Variant {
    fn from(i: i64) -> Variant {
        Variant::VLong(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON_SINGLE: f32 = 0.00001;
    const EPSILON_DOUBLE: f64 = 0.00001;

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

            fn test_rounding_with_f32(left: i32, right: f32, expected: i32) {
                match Variant::VInteger(left).plus(&Variant::VSingle(right)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_single() {
                test_rounding_with_f32(1, 0.0, 1);
                test_rounding_with_f32(1, 0.1, 1);
                test_rounding_with_f32(1, 0.5, 2);
                test_rounding_with_f32(1, 0.9, 2);
                test_rounding_with_f32(1, -0.5, 0);
                test_rounding_with_f32(-42, -2.1, -44);
                test_rounding_with_f32(-42, -2.5, -45);
                test_rounding_with_f32(-42, -2.75, -45);
            }

            fn test_rounding_with_f64(left: i32, right: f64, expected: i32) {
                match Variant::VInteger(left).plus(&Variant::VDouble(right)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                test_rounding_with_f64(1, 0.0, 1);
                test_rounding_with_f64(1, 0.1, 1);
                test_rounding_with_f64(1, 0.5, 2);
                test_rounding_with_f64(1, 0.9, 2);
                test_rounding_with_f64(1, -0.5, 0);
                test_rounding_with_f64(-42, -2.1, -44);
                test_rounding_with_f64(-42, -2.5, -45);
                test_rounding_with_f64(-42, -2.75, -45);
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

            fn test_rounding_with_f32(left: i64, right: f32, expected: i64) {
                match Variant::VLong(left).plus(&Variant::VSingle(right)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_single() {
                test_rounding_with_f32(1, 0.0, 1);
                test_rounding_with_f32(1, 0.1, 1);
                test_rounding_with_f32(1, 0.5, 2);
                test_rounding_with_f32(1, 0.9, 2);
                test_rounding_with_f32(1, -0.5, 0);
                test_rounding_with_f32(-42, -2.1, -44);
                test_rounding_with_f32(-42, -2.5, -45);
                test_rounding_with_f32(-42, -2.75, -45);
            }

            fn test_rounding_with_f64(left: i64, right: f64, expected: i64) {
                match Variant::VLong(left).plus(&Variant::VDouble(right)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                test_rounding_with_f64(1, 0.0, 1);
                test_rounding_with_f64(1, 0.1, 1);
                test_rounding_with_f64(1, 0.5, 2);
                test_rounding_with_f64(1, 0.9, 2);
                test_rounding_with_f64(1, -0.5, 0);
                test_rounding_with_f64(-42, -2.1, -44);
                test_rounding_with_f64(-42, -2.5, -45);
                test_rounding_with_f64(-42, -2.75, -45);
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

            fn test_rounding_with_f32(left: i32, right: f32, expected: i32) {
                match Variant::VInteger(left).minus(&Variant::VSingle(right)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_single() {
                test_rounding_with_f32(1, 0.0, 1);
                test_rounding_with_f32(1, 0.1, 1);
                test_rounding_with_f32(1, 0.5, 0);
                test_rounding_with_f32(1, 0.9, 0);
                test_rounding_with_f32(1, -0.5, 2);
            }

            fn test_rounding_with_f64(left: i32, right: f64, expected: i32) {
                match Variant::VInteger(left).minus(&Variant::VDouble(right)).unwrap() {
                    Variant::VInteger(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                test_rounding_with_f64(1, 0.0, 1);
                test_rounding_with_f64(1, 0.1, 1);
                test_rounding_with_f64(1, 0.5, 0);
                test_rounding_with_f64(1, 0.9, 0);
                test_rounding_with_f64(1, -0.5, 2);
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

            fn test_rounding_with_f32(left: i64, right: f32, expected: i64) {
                match Variant::VLong(left).minus(&Variant::VSingle(right)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_single() {
                test_rounding_with_f32(1, 0.0, 1);
                test_rounding_with_f32(1, 0.1, 1);
                test_rounding_with_f32(1, 0.5, 0);
                test_rounding_with_f32(1, 0.9, 0);
                test_rounding_with_f32(1, -0.5, 2);
            }

            fn test_rounding_with_f64(left: i64, right: f64, expected: i64) {
                match Variant::VLong(left).minus(&Variant::VDouble(right)).unwrap() {
                    Variant::VLong(result) => assert_eq!(result, expected),
                    _ => panic!("assertion failed"),
                }
            }

            #[test]
            fn test_double() {
                test_rounding_with_f64(1, 0.0, 1);
                test_rounding_with_f64(1, 0.1, 1);
                test_rounding_with_f64(1, 0.5, 0);
                test_rounding_with_f64(1, 0.9, 0);
                test_rounding_with_f64(1, -0.5, 2);
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
}
