use crate::interpreter::argument::Argument;
use crate::linter::ParamName;
use std::collections::{HashMap, VecDeque};

pub enum Arguments {
    Named(HashMap<ParamName, Argument>),
    Unnamed(VecDeque<Argument>),
}

impl Arguments {
    pub fn named() -> Self {
        Self::Named(HashMap::new())
    }

    pub fn unnamed() -> Self {
        Self::Unnamed(VecDeque::new())
    }

    pub fn get(&self, param_name: &ParamName) -> Option<&Argument> {
        match self {
            Self::Named(map) => map.get(param_name),
            Self::Unnamed(_) => None,
        }
    }

    pub fn get_mut(&mut self, param_name: &ParamName) -> Option<&mut Argument> {
        match self {
            Self::Named(map) => map.get_mut(param_name),
            Self::Unnamed(_) => None,
        }
    }

    pub fn push_unnamed<T>(&mut self, arg: T)
    where
        Argument: From<T>,
    {
        match self {
            Self::Named(_) => panic!("Cannot push unnamed in Arguments::Named"),
            Self::Unnamed(v) => v.push_back(arg.into()),
        }
    }

    pub fn push_named<T>(&mut self, parameter_name: ParamName, arg: T)
    where
        Argument: From<T>,
    {
        match self {
            Self::Named(m) => match m.insert(parameter_name, arg.into()) {
                Some(_) => panic!("Duplicate key!"),
                None => {}
            },
            Self::Unnamed(_) => panic!("Cannot push named in Arguments::Unnamed"),
        }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<Argument> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.iter(),
        }
    }

    pub fn get_at(&self, index: usize) -> Option<&Argument> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.get(index),
        }
    }
}
