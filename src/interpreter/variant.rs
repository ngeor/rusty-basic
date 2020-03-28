use crate::common::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    VFloat(f32),
    VDouble(f64),
    VString(String),
    VInteger(i32),
    VLong(i64),
}

impl Variant {
    pub fn is_true(&self) -> Result<bool> {
        match self {
            Variant::VFloat(n) => Ok(*n != 0.0),
            Variant::VDouble(n) => Ok(*n != 0.0),
            Variant::VString(s) => Err("Type mismatch".to_string()),
            Variant::VInteger(n) => Ok(*n != 0),
            Variant::VLong(n) => Ok(*n != 0),
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Variant::VFloat(n) => format!("{}", n),
            Variant::VDouble(n) => format!("{}", n),
            Variant::VString(s) => s.clone(),
            Variant::VInteger(n) => format!("{}", n),
            Variant::VLong(n) => format!("{}", n),
        }
    }

    pub fn to_int(&self) -> Result<i32> {
        match self {
            Variant::VFloat(f) => Ok(f.round() as i32),
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

    pub fn plus(&self, other: &Variant) -> Variant {
        match self {
            Variant::VString(s_left) => Variant::VString(format!("{}{}", s_left, other.to_str())),
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Variant::VInteger(*i_left + *i_right),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn minus(&self, other: &Variant) -> Result<Variant> {
        match self {
            Variant::VString(_) => Err(format!("Operator - not applicable to strings")),
            Variant::VInteger(i_left) => match other {
                Variant::VInteger(i_right) => Ok(Variant::VInteger(*i_left - *i_right)),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

impl From<f32> for Variant {
    fn from(f: f32) -> Variant {
        Variant::VFloat(f)
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
