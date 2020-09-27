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

    pub fn get(&self, param_name: &ParamName) -> Option<&Variant> {
        match self {
            Self::Named(map) => map.get(param_name),
            Self::Unnamed(_) => None,
        }
    }

    pub fn get_mut(&mut self, param_name: &ParamName) -> Option<&mut Variant> {
        match self {
            Self::Named(map) => map.get_mut(param_name),
            Self::Unnamed(_) => None,
        }
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

    pub fn iter(&self) -> std::collections::vec_deque::Iter<Variant> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.iter(),
        }
    }

    pub fn get_at(&self, index: usize) -> Option<&Variant> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.get(index),
        }
    }
}
