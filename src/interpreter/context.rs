use crate::common::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    VString(String),
    VNumber(i32),
}

impl Variant {
    pub fn is_true(&self) -> bool {
        match self {
            Variant::VString(s) => s.len() > 0,
            Variant::VNumber(i) => *i != 0,
        }
    }

    pub fn compare_to(&self, other: &Variant) -> Result<std::cmp::Ordering> {
        match self {
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Ok(s_left.cmp(s_right)),
                Variant::VNumber(i_right) => {
                    let tmp = format!("{}", i_right);
                    Ok(s_left.cmp(&tmp))
                }
            },
            Variant::VNumber(i_left) => match other {
                Variant::VString(s_right) => {
                    let tmp = format!("{}", i_left);
                    Ok(tmp.cmp(s_right))
                }
                Variant::VNumber(i_right) => Ok(i_left.cmp(i_right)),
            },
        }
    }

    pub fn plus(&self, other: &Variant) -> Variant {
        match self {
            Variant::VString(s_left) => match other {
                Variant::VString(s_right) => Variant::VString(format!("{}{}", s_left, s_right)),
                Variant::VNumber(i_right) => Variant::VString(format!("{}{}", s_left, i_right)),
            },
            Variant::VNumber(i_left) => match other {
                Variant::VString(s_right) => Variant::VString(format!("{}{}", i_left, s_right)),
                Variant::VNumber(i_right) => Variant::VNumber(*i_left + *i_right),
            },
        }
    }

    pub fn minus(&self, other: &Variant) -> Result<Variant> {
        match self {
            Variant::VString(s_left) => Err(format!("Operator - not applicable to strings")),
            Variant::VNumber(i_left) => match other {
                Variant::VString(s_right) => Err(format!("Operator - not applicable to strings")),
                Variant::VNumber(i_right) => Ok(Variant::VNumber(*i_left - *i_right)),
            },
        }
    }
}

/// A variable context
#[derive(Debug, Clone)]
pub struct Context {
    variable_map: HashMap<String, Variant>,
}

pub trait ReadOnlyContext {
    fn get_variable<S: AsRef<str>>(&self, variable_name: S) -> Result<Variant>;
}

pub trait ReadWriteContext: ReadOnlyContext {
    fn set_variable(&mut self, variable_name: String, variable_value: Variant) -> Result<()>;
}

impl Context {
    pub fn new() -> Context {
        Context {
            variable_map: HashMap::new(),
        }
    }
}

impl ReadOnlyContext for Context {
    fn get_variable<S: AsRef<str>>(&self, variable_name: S) -> Result<Variant> {
        match self.variable_map.get(variable_name.as_ref()) {
            Some(v) => Ok(v.clone()),
            None => Err(format!(
                "Variable {} is not defined",
                variable_name.as_ref()
            )),
        }
    }
}

impl ReadWriteContext for Context {
    fn set_variable(&mut self, variable_name: String, variable_value: Variant) -> Result<()> {
        self.variable_map.insert(variable_name, variable_value);
        Ok(())
    }
}
