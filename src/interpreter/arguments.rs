use crate::linter::ParamName;
use crate::variant::Variant;
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub enum Arguments {
    Named(HashMap<ParamName, Variant>),
    Unnamed(VecDeque<Variant>),
}

impl Arguments {
    pub fn named() -> Self {
        Self::Named(HashMap::new())
    }

    pub fn unnamed() -> Self {
        Self::Unnamed(VecDeque::new())
    }

    pub fn push_unnamed(&mut self, arg: Variant) {
        match self {
            Self::Named(_) => panic!("Cannot push unnamed in Arguments::Named"),
            Self::Unnamed(v) => v.push_back(arg),
        }
    }

    pub fn push_named(&mut self, parameter_name: ParamName, arg: Variant) {
        match self {
            Self::Named(m) => match m.insert(parameter_name, arg) {
                Some(_) => panic!("Duplicate key!"),
                None => {}
            },
            Self::Unnamed(_) => panic!("Cannot push named in Arguments::Unnamed"),
        }
    }
}
